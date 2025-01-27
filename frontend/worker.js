import init, { paniky, set_panic_hook_js, ExampleStruct, TreeDataWrapper } from './wasm_hashlife.js';

async function run() {
    await init();

    // const set_panic_hook_js = exports.set_panic_hook_js;
    // const TreeDataWrapper = exports.TreeDataWrapper;
    // import {paniky, set_panic_hook_js, ExampleStruct, TreeDataWrapper} from "wasm-game-of-life";
    set_panic_hook_js();
    var tree = TreeDataWrapper.new();
    var hashsize_limit = 1e40;

    self.onmessage = function(e) {
        const workerData = e.data;
        // console.log("Message from main")
        // console.log(workerData.type)
        if (workerData.type === "set_rle"){
            tree.free();
            tree = TreeDataWrapper.make_from_rle(workerData.data);
        }
        else if (workerData.type == "step_forward"){
            tree.step_forward(workerData.amount)
            if (tree.hash_count() > 0.9*hashsize_limit){
                console.log("worker collectgin garbage")
                console.log("prev size", tree.hash_count())
                const newtree = tree.pruned_tree();
                tree.free();
                tree = newtree;
                console.log("fin size", tree.hash_count())
            }
            var pruned_tree = tree.pruned_tree();
            var serialized = pruned_tree.serialize_treerepr();
            self.postMessage({
                type: "serialized_tree",
                data: serialized,
                hash_count: tree.hash_count(),
            });
            pruned_tree.free();
        }
        else if (workerData.type == "set_garbage_limit"){
            hashsize_limit = workerData.amount;
            if (tree.hash_count() > 0.9*hashsize_limit){
                const newtree = tree.pruned_tree();
                tree.free();
                tree = newtree;
            }
        }
        else{
            console.log("unknown type: "+workerData.type);
        }
    }
    self.postMessage({
        type: "ready"
    })
}
run()