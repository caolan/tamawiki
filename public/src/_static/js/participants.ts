import "@webcomponents/custom-elements";
import "../css/participants.css";
import * as protocol from "./protocol";

export class ParticipantsElement extends HTMLElement {
    private participants: protocol.Participant[];
    private ul?: HTMLUListElement;
    private localId?: number;

    constructor() {
        super();
        this.participants = [];
    }

    public connectedCallback(): void {
        if (!this.ul) {
            this.ul = document.createElement("ul");
            this.appendChild(this.ul);
        }
        this.renderList();
    }

    public addParticipant(participant: protocol.Participant): void {
        this.participants.push(participant);
        this.renderList();
    }

    public removeParticipant(id: number): void {
        this.participants = this.participants.filter((p) => {
            return p.id !== id;
        });
        this.renderList();
    }

    public setParticipants(data: protocol.Participant[]): void {
        this.participants = data;
        this.renderList();
    }

    public setLocalParticipantId(id: number): void {
        this.localId = id;
        this.addParticipant(new protocol.Participant(id, 0));
        this.renderList();
    }

    public getLocalParticipantId(): number | undefined {
        return this.localId;
    }

    private renderList(): void {
        if (this.ul) {
            this.ul.innerHTML = "";
            for (const participant of this.participants) {
                const li = document.createElement("li");
                li.textContent = "Participant " + participant.id;
                if (participant.id === this.localId) {
                    li.className = "you";
                }
                this.ul.appendChild(li);
            }
        }
    }
}

customElements.define("tw-participants", ParticipantsElement);
