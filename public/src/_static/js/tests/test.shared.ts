import { assert } from "chai";
import { Event } from "../protocol";
import testcases from "./shared/transform.json";

suite("shared/transform.json", function() {
    for (const t of testcases) {
        test(t.name as string, () => {
            const event = Event.fromJSON(t.initial);
            const concurrent = (t.concurrent as any[]).map(Event.fromJSON);
            for (const c of concurrent) {
                event.transform(c);
            }
            assert.deepEqual(
                JSON.stringify(event.toJSON()),
                JSON.stringify(t.expected),
            );
        });
    }
});
