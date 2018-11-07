import { assert } from "chai";
import { ContentElement } from "../content";
import * as protocol from "../protocol";
import applyTests from "./shared/apply.json";
import transformTests from "./shared/transform.json";

suite("shared/apply.json", function() {
    setup(function() {
        this.tmp = document.createElement("div");
        document.body.appendChild(this.tmp);
    });

    teardown(function() {
        document.body.removeChild(this.tmp);
    });

    for (const t of applyTests) {
        test(t.name as string, function() {
            const doc = protocol.Document.fromJSON(t.initial);
            const events = (t.events as any[]).map(protocol.Event.fromJSON);
            let seq = 3;

            // setup ContentElement
            const content = new ContentElement();
            this.tmp.appendChild(content);
            content.loadDocument(seq, doc);

            // apply the events
            for (const ev of events) {
                seq++;
                try {
                    content.applyEvent(seq, ev);
                    // make sure we were not expecting an error
                    assert.equal(t.error, null);
                } catch (e) {
                    // capture the error if we were expecting one
                    if (t.error) {
                        assert.equal(e.toString(), "Error: " + t.error.type);
                    } else {
                        throw e;
                    }
                }
            }

            // check the state of the document after applying the events
            const expected = protocol.Document.fromJSON(t.expected);
            assert.equal(content.getValue(), expected.content);
            assert.equal(
                Object.keys(content.participants).length,
                expected.participants.length,
            );
            for (const p of expected.participants) {
                assert.equal(
                    content.getParticipantPosition(p.id),
                    p.cursor_pos,
                );
            }
        });
    }
});

suite("shared/transform.json", function() {
    for (const t of transformTests) {
        test(t.name as string, function() {
            const event = protocol.Event.fromJSON(t.initial);
            const concurrent = (t.concurrent as any[]).map(protocol.Event.fromJSON);
            for (const c of concurrent) {
                event.transform(c);
            }
            assert.deepEqual(
                JSON.stringify(event.toJSON()),
                JSON.stringify(t.expected),
            );
        });
    }
});
