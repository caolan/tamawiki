//! Handles client communication with a DocumentSession
use futures::future::Future;
use futures::sink::Sink;
use futures::stream::{self, Stream};
use futures::task;
use futures::{Async, AsyncSink, Poll, StartSend};
use std::mem;
use tokio::executor::{DefaultExecutor, Executor};

use super::message::*;
use super::DocumentSession;
use document::{Edit, Event, Join, Leave, ParticipantId};
use store::{SequenceId, Store, StoreError};

/// A client connected to a DocumentSession. This struct can be used
/// as a Stream, to read ServerMessages applicable to the participant,
/// and a Sink, for the participant to send ClientMessages to the
/// DocumentSession.
pub struct Participant<T: Store + Sync> {
    id: ParticipantId,
    seq: SequenceId,
    client_seq: SequenceId,
    session: DocumentSession<T>,
    state: ParticipantStreamState,
    inner: stream::Flatten<stream::FuturesUnordered<T::SinceFuture>>,
    writing: Option<(
        SequenceId,
        Box<Future<Item = SequenceId, Error = StoreError> + Send>,
    )>,
}

#[derive(Debug)]
enum ParticipantStreamState {
    ReadingStream,
    WaitingForEvent,
}

impl<T: Store + Sync> Participant<T> {
    /// Creates a new Participant for the given DocumentSession. You
    /// should not need to call this directly, instead a Participant
    /// should be obtained by calling `join()` using the
    /// DocuemntSessionManager.
    pub fn new(session: DocumentSession<T>, id: ParticipantId, since: SequenceId) -> Self {
        let catchup = {
            let data = session.data.lock().unwrap();
            data.store.since(&data.path.as_path(), since)
        };
        Self {
            session: session,
            seq: since,
            client_seq: 0,
            state: ParticipantStreamState::ReadingStream,
            inner: stream::futures_unordered(vec![catchup]).flatten(),
            writing: None,
            id,
        }
    }

    /// Returns the ParticipantId.
    pub fn get_id(&self) -> ParticipantId {
        self.id
    }

    // If this function returns true, the event will not be converted
    // to a ServerMessage and sent to the Participant when it reads
    // the next event.
    fn ignored_event(&self, event: &Event) -> bool {
        match event {
            &Event::Edit(Edit { author, .. }) => author == self.id,
            &Event::Leave(Leave { id }) => id == self.id,
            &Event::Join(Join { id }) => id == self.id,
        }
    }

    // Converts an event into a ServerMessage, adding the server's
    // sequence id for the event and the most recently applied client
    // sequence id from the participant.
    fn prepare_server_message(&self, seq: SequenceId, event: Event) -> ServerMessage {
        ServerMessage::Event(ServerEventMessage {
            client_seq: self.client_seq,
            seq,
            event
        })
    }
}

impl<T: Store + Sync> Stream for Participant<T> {
    type Item = ServerMessage;
    type Error = MessageStreamError;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                ParticipantStreamState::ReadingStream => match self.inner.poll()? {
                    Async::Ready(None) => {
                        self.state = ParticipantStreamState::WaitingForEvent;
                    }
                    Async::Ready(Some((seq, event))) => {
                        self.seq = seq;
                        if !self.ignored_event(&event) {
                            let msg = self.prepare_server_message(seq, event);
                            return Ok(Async::Ready(Some(msg)));
                        }
                    }
                    Async::NotReady => return Ok(Async::NotReady),
                },
                ParticipantStreamState::WaitingForEvent => {
                    let last_seq = {
                        let data = self.session.data.lock().unwrap();
                        data.last_seq
                    };
                    match last_seq {
                        Some(last_seq) if last_seq > self.seq => {
                            self.inner = {
                                let data = self.session.data.lock().unwrap();
                                stream::futures_unordered(vec![
                                    data.store.since(&data.path.as_path(), self.seq),
                                ]).flatten()
                            };
                            self.state = ParticipantStreamState::ReadingStream;
                        }
                        _ => {
                            self.session.notify_on_event(task::current());
                            return Ok(Async::NotReady);
                        }
                    }
                }
            }
        }
    }
}

impl<T: Store + Sync> Sink for Participant<T> {
    type SinkItem = ClientMessage;
    type SinkError = MessageStreamError;

    fn start_send(&mut self, item: Self::SinkItem) -> StartSend<Self::SinkItem, Self::SinkError> {
        loop {
            let writing = mem::replace(&mut self.writing, None);
            match writing {
                None => match item {
                    ClientMessage::ClientEdit(data) => {
                        let event = Event::Edit(Edit {
                            author: self.id,
                            operations: data.operations,
                        });
                        self.writing = Some((
                            data.client_seq,
                            Box::new(self.session.write_transformed(
                                self.id,
                                data.parent_seq,
                                event,
                            )),
                        ));
                        return Ok(AsyncSink::Ready);
                    }
                },
                Some((client_seq, mut push_future)) => match push_future.poll()? {
                    Async::NotReady => {
                        self.writing = Some((client_seq, push_future));
                        return Ok(AsyncSink::NotReady(item));
                    }
                    Async::Ready(_) => {
                        self.client_seq = client_seq;
                        return Ok(AsyncSink::Ready);
                    }
                },
            }
        }
    }

    fn poll_complete(&mut self) -> Poll<(), Self::SinkError> {
        let writing = mem::replace(&mut self.writing, None);
        match writing {
            None => Ok(Async::Ready(())),
            Some((client_seq, mut push_future)) => match push_future.poll()? {
                Async::NotReady => {
                    self.writing = Some((client_seq, push_future));
                    Ok(Async::NotReady)
                }
                Async::Ready(_) => {
                    self.client_seq = client_seq;
                    self.writing = None;
                    Ok(Async::Ready(()))
                }
            },
        }
    }
}

impl<T: Store + Sync> Drop for Participant<T> {
    fn drop(&mut self) {
        let id = self.id;
        let result = DefaultExecutor::current().spawn(Box::new(
            self.session.leave(id).map(|_| ()).map_err(move |err| {
                eprintln!(
                    "Error when participant {} leaving document session: {}",
                    id, err
                );
            }),
        ));
        // ignore error spawning future for leave notifications if the
        // current executor is shutting down
        if let Err(err) = result {
            if !err.is_shutdown() {
                panic!("{:?}", err);
            }
        }
    }
}
