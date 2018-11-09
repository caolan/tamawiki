import "@webcomponents/custom-elements";
import { IConnectionConstructor, WebSocketConnection } from "./connection";
import { ContentElement } from "./content";
import { ParticipantsElement } from "./participants";
import * as protocol from "./protocol";
import { Session } from "./session";

import "../css/editor.css";

export class Editor extends HTMLElement {
    public ConnectionType: IConnectionConstructor;
    public content: ContentElement;
    public participantId?: number;
    public participants: ParticipantsElement;
    public session?: Session;

    /**
     * @param ConnnectionType  The class to use when creating a new connection
     */
    constructor(ConnectionType?: IConnectionConstructor) {
        super();
        this.content = new ContentElement();
        this.participants = new ParticipantsElement();
        this.ConnectionType = ConnectionType || WebSocketConnection;
    }

    connectedCallback(): void {
        const seq = Number(this.getAttribute("initial-seq") || "0");
        const participantsData = JSON.parse(this.getAttribute("participants") || "[]");
        const text = this.textContent || "";
        this.textContent = "";

        this.session = new Session(
            new this.ConnectionType(window.location.pathname, seq)
        );

        // initialize participants window
        this.participants.setParticipants(participantsData);
        this.appendChild(this.participants);
        this.session.on("connected", (id) => {
            this.participants.setLocalParticipantId(id);
        });
        this.session.on("join", (seq, participant) => {
            this.participants.addParticipant(participant);
            this.content.addParticipant(seq, participant);
        });
        this.session.on("leave", (seq, id) => {
            this.participants.removeParticipant(id);
            this.content.removeParticipant(seq, id);
        });
        this.session.on("edit", (seq, event) => {
            // NOTE: this must be applied synchronously, as the
            // transforms applied on the server event are for the
            // *current* state of the editor.
            this.content.applyEvent(seq, event);
        });

        // initialize content editor
        this.appendChild(this.content);
        this.content.loadDocument(
            seq,
            new protocol.Document(text, participantsData),
        );
        // TODO: how to handle change events that occur before
        // Connected message arrives?
        this.content.events.on(
            "change",
            (parentSeq: number, operations: protocol.Operation[]) => {
                (this.session as Session).send(parentSeq, operations);
            }
        );
    }
}

customElements.define("tw-editor", Editor);
