import * as CodeMirror from "codemirror";
import "codemirror/lib/codemirror.css";

export class Content extends HTMLElement {
    public codemirror?: CodeMirror.Editor;
    
    constructor() {
        super();
    }
    
    connectedCallback(): void {
        if (!this.codemirror) {
            this.codemirror = CodeMirror(this, {
                lineWrapping: true,
                mode: "text",
                value: this.textContent,
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

customElements.define("tw-content", Content);
