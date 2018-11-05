import "@webcomponents/custom-elements";
import * as protocol from "./protocol";

export class ParticipantsElement extends HTMLElement {
    private participants: protocol.Participant[];
    private ul?: HTMLUListElement;
    private localId?: number;

    constructor() {
        super();
        this.participants = [];
    }

    connectedCallback(): void {
        if (!this.ul) {
            this.ul = document.createElement("ul");
            this.appendChild(this.ul);
        }
        this.renderList();
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

    addParticipant(participant: protocol.Participant): void {
        this.participants.push(participant);
        this.renderList();
    }

    setParticipants(data: protocol.Participant[]): void {
        this.participants = data;
        this.renderList();
    }

    setLocalParticipantId(id: number): void {
        this.localId = id;
        this.addParticipant(new protocol.Participant(id, 0));
        this.renderList();
    }

    getLocalParticipantId(): number | undefined {
        return this.localId;
    }
}

customElements.define("tw-participants", ParticipantsElement);
