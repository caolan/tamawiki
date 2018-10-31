import "@webcomponents/custom-elements";
import * as CodeMirror from "codemirror";
import "codemirror/lib/codemirror.css";

export class ContentElement extends HTMLElement {
    public codemirror?: CodeMirror.Editor;
    
    constructor() {
        super();
    }
    
    connectedCallback(): void {
        if (!this.codemirror) {
            const value = this.textContent;
            this.textContent = "";
            this.codemirror = CodeMirror(this, {
                lineWrapping: true,
                mode: "text",
                value: value,
            });
        }
    }

    getValue(): string {
        if (this.codemirror) {
            return this.codemirror.getDoc().getValue();
        }
        return "";
    }
}

customElements.define("tw-content", ContentElement);
