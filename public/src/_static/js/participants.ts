import "@webcomponents/custom-elements";
import * as protocol from "./protocol";

export class ParticipantsElement extends HTMLElement {
    private participants: protocol.Participant[];
    private ul?: HTMLUListElement;
    
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
            for (const participant of this.participants) {
                const li = document.createElement("li");
                li.textContent = "Participant " + participant.id;
                this.ul.appendChild(li);
            }
        }
    }

    setParticipants(data: protocol.Participant[]): void {
        this.participants = data;
        this.renderList();
    }
}

customElements.define("tw-participants", ParticipantsElement);
