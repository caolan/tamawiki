import { Content } from "./content";
import "../css/editor.css"

export class Editor extends HTMLElement {
    public content: Content;
    
    constructor() {
        super();
        this.content = new Content();
    }

    connectedCallback(): void {
        this.content.textContent = this.textContent;
        this.textContent = "";
        this.appendChild(this.content);
    }
}

customElements.define("tw-editor", Editor);
