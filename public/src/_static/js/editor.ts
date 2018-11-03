import "@webcomponents/custom-elements";
import { Duplex } from "stream";
import { websocketConnect, ConnectFunction } from "./connection";
import { ContentElement } from "./content";
import { ParticipantsElement } from "./participants";
import * as protocol from "./protocol";
import { Session } from "./session";

import "../css/editor.css";

export class Editor extends HTMLElement {
    public connect: ConnectFunction;
    public content: ContentElement;
    public connection?: Duplex;
    public participants: ParticipantsElement;
    public session?: Session;

    /**
     * @param Conn  The class to use when creating a new connection
     */
    constructor(connect?: ConnectFunction) {
        super();
        this.content = new ContentElement();
        this.participants = new ParticipantsElement();
        this.connect = connect || websocketConnect;
    }

    connectedCallback(): void {
        const seq = Number(this.getAttribute("initial-seq") || "0");
        const participantsData = JSON.parse(this.getAttribute("participants") || "[]");
        const text = this.textContent || "";
        this.textContent = "";

        // initialize participants window
        this.participants.setParticipants(participantsData);
        this.appendChild(this.participants);

        // initialize content editor
        this.appendChild(this.content);
        this.content.loadDocument(new protocol.Document(text, []));

        // start connection
        this.connection = this.connect(window.location.pathname, seq);
        this.session = new Session(seq);
    }
}

customElements.define("tw-editor", Editor);
