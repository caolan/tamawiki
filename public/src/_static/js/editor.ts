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
            new this.ConnectionType(window.location.pathname, seq),
            seq,
        );

        // initialize participants window
        this.participants.setParticipants(participantsData);
        this.appendChild(this.participants);

        // initialize content editor
        this.appendChild(this.content);
        this.content.loadDocument(new protocol.Document(text, []));
    }
}

customElements.define("tw-editor", Editor);
