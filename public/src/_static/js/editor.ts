import "@webcomponents/custom-elements";
import { Connection, IConnectionConstructor } from "./connection";
import { ContentElement } from "./content";
import { ParticipantsElement } from "./participants";
import { Session } from "./session";

import "../css/editor.css";

export class Editor extends HTMLElement {
    public Conn: IConnectionConstructor;
    public content: ContentElement;
    public connection?: Connection;
    public participants: ParticipantsElement;
    public session?: Session;
    
    constructor(Conn?: IConnectionConstructor) {
        super();
        this.content = new ContentElement();
        this.participants = new ParticipantsElement();
        this.Conn = Conn || Connection;
    }

    connectedCallback(): void {
        const seq = Number(this.getAttribute("initial-seq") || "0");
        const participantsData = JSON.parse(this.getAttribute("participants") || "[]");
        const text = this.textContent;
        this.textContent = "";
        
        // initialize participants window
        this.participants.setParticipants(participantsData);
        this.appendChild(this.participants);
        
        // initialize content editor
        this.content.textContent = text;
        this.appendChild(this.content);
        
        // start connection
        this.connection = new this.Conn(window.location.pathname, seq);
        this.session = new Session(seq);
    }
}

customElements.define("tw-editor", Editor);
