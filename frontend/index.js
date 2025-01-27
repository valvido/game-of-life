import init, { paniky, set_panic_hook_js, ExampleStruct, TreeDataWrapper } from './wasm_hashlife.js';

async function run() {
    await init();


var filename = "example_spaceship.rle";
const RLE_STR = (
    "x = 12, y = 8, rule = B3/S23\n" +
    "5bob2o$4bo6bo$3b2o3bo2bo$2obo5b2o$2obo5b2o$3b2o3bo2bo$4bo6bo$5bob2o!\n"
);
var tree = TreeDataWrapper.make_from_rle(RLE_STR);
var workerhashcount = 0;
var canvas = document.getElementById("game-of-life-canvas");
var xsize = window.innerWidth;
var ysize = window.innerHeight;
var xstart = 0;
var ystart = 0;
var brightnessSelect = document.getElementById("brightness-select");
var garbageSelect = document.getElementById("garbage-select");
var filedata = RLE_STR;
var xyfilecoord = [12,8];
var inputFileLoader = document.getElementById("rle-file-input");
var resetBoundingButton = document.getElementById("reset-bounding-box")
var downloadButton = document.getElementById("download-rle")
var examplesSelect = document.getElementById("examples-select")
var play_pause = document.getElementById("play-pause")
var zoom_level = -1;
const myWorker = new Worker("worker.js", {type: "module"});
var last_step_time = 0;
var needs_render = true;
var current_speed = 1;
var current_fps = 4;
var is_paused = false;


document.getElementById("decrease-speed").onclick = function(){
    current_speed = Math.max(current_speed - 1, 1);
    render();
}
document.getElementById("increase-speed").onclick = function(){
    current_speed = current_speed + 1;
    render();
}
document.getElementById("fast-backwards").onclick = function(){
    current_fps = Math.max(0, current_fps - 1);
    render();
}
document.getElementById("fast-forwards").onclick = function(){
    current_fps = Math.min(12, current_fps + 1);
    render();
}
function cellSize(){
    return Math.pow(2, Math.floor((zoom_level < 0 ? -zoom_level : 0)))//cellSizeSelect.value)
}
function zoomLevel(){
    return Math.max(0,Math.floor(zoom_level))
}
function zoomScale(){
    return Math.pow(2, zoomLevel())/cellSize() 
}
function brightness(){
    return Math.pow(2,0.25*brightnessSelect.value)
}
function roundToCell(size){
    return Math.ceil(size/cellSize())*cellSize()
}
function garbageLimit(){
    return Math.pow(2,garbageSelect.value)
}
function actualRender(){
    requestAnimationFrame(actualRender);
    if (!needs_render){
        return false;
    }
    needs_render = false;
    // console.log("rendered!")
    // console.log(tree.hash_count());
    // console.log(tree.num_live_cells());
    var map = tree.make_grayscale_map(xstart,ystart,Math.ceil(xsize/cellSize()), Math.ceil(ysize/cellSize()),cellSize(),zoomLevel(),brightness());
    // console.log(map);
    var clamped_data = new Uint8ClampedArray(map);
    // console.log(clamped_data);
    var img_data = new ImageData(clamped_data,roundToCell(xsize),roundToCell(ysize));
    // console.log(img_data);
    canvas.width = xsize;
    canvas.height = ysize;
    const canvasContext = canvas.getContext("2d");
    canvasContext.imageSmoothingEnabled = false;
    canvasContext.putImageData(img_data, 0, 0);

    document.getElementById("cell-count").innerText = tree.num_live_cells();
    document.getElementById("cached-hash-count").innerText = workerhashcount;
    document.getElementById("static-hash-count").innerText = tree.hash_count();
    document.getElementById("universe-age").innerText = tree.get_age();
    document.getElementById("max-memory-display").innerText = garbageLimit()*(112+8*3)/Math.pow(2,20);
    document.getElementById("brightness-display").innerText = Math.round(100*brightness())/100;
    document.getElementById("steps-per-frame").innerText = Math.pow(2,current_speed);
    document.getElementById("frames-per-second").innerText = Math.round(100*Math.pow(2,current_fps/2.))/100;
    document.getElementById("zoom-ratio").innerText = zoomScale();
}
function render(){
    needs_render = true;
}
function clearCanvas(){
    var ctx = canvas.getContext('2d');
    
    // fill the entire canvas with black before drawing the circles
    ctx.fillStyle='black';
    ctx.fillRect(0,0,canvas.width,canvas.height);
}
const renderLoop = () => {
    const desired_interval = 1000/Math.pow(2,current_fps/2.);
    const cur_step_time = new Date().getTime();
    if (!is_paused && cur_step_time - last_step_time > desired_interval-5){
        myWorker.postMessage({
            type: "step_forward",
            amount: Math.pow(2,current_speed),
        });
        last_step_time = cur_step_time;
    }
    else{
        setTimeout(renderLoop, desired_interval - (cur_step_time - last_step_time));
    }
};

function bound_zoom(zoom_level){
    zoom_level = Math.max(zoom_level, -5);
    zoom_level = Math.min(zoom_level, 15);
    return zoom_level;
}
function handle_wheel(event){
    // console.log(event);
    var oldscale = zoomScale();
    var cenx = xstart + event.offsetX*oldscale;
    var ceny = ystart + event.offsetY*oldscale;
    zoom_level -= event.deltaY*0.03;
    zoom_level = bound_zoom(zoom_level)
    var newscale = zoomScale();
    xstart = cenx - event.offsetX*newscale;
    ystart = ceny - event.offsetY*newscale;
    render();
    event.stopPropagation();
}
function handleRleUpdate(filedata){
    myWorker.postMessage({
        type: "set_rle",
        data: filedata,
    });
    //make sure to keep a local copy at all times
    tree.free()
    tree = TreeDataWrapper.make_from_rle(filedata);
    parseBoundingBox(filedata)
    resetBoundingBox()
    current_speed = 1;
    current_fps = 4;
    render();
}
function handleFileUpload() {
    clearCanvas();
    var file = inputFileLoader.files[0];
    filename = file.name;
    const reader = new FileReader();
    reader.onload = function(){
        handleRleUpdate(reader.result);
    }
    reader.readAsText(file);
}
function parseBoundingBox(filedata){
    var boundsline = filedata.split('\n').filter((l)=>l[0] != "#")[0]
    const numregex = /\d+/g;
    xyfilecoord = boundsline.match(numregex);
}
function resetBoundingBox(){
    var filex = xyfilecoord[0];
    var filey = xyfilecoord[1];
    // console.log(filex);
    // console.log(filey);
    xstart = -filex/4;
    ystart = -filey/4;
    var zoomx = Math.log2(filex*2 / canvas.width);
    var zoomy = Math.log2(filey*2 / canvas.height);
    // console.log(zoomx);
    // console.log(zoomy);
    zoom_level = bound_zoom(Math.max(zoomx,zoomy));
    var scale = zoomScale();
    let xcen = scale*canvas.width/2;
    let ycen = scale*canvas.height/2;
    xstart = filex / 2 - xcen;
    ystart = filey / 2 - ycen;
    brightnessSelect.value = Math.max(1,Math.floor(zoom_level)*4)
    render()
}
function downloadText(text, filename){
  var element = document.createElement('a');
  element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(text));
  element.setAttribute('download', filename);

  element.style.display = 'none';
  document.body.appendChild(element);

  element.click();

  document.body.removeChild(element);
}
function downloadRLE(){
    downloadText(tree.get_rle(),filename+"."+tree.get_age());
}
function onWindowResize(event){
    xsize = window.innerWidth;
    ysize = window.innerHeight;
    canvas.width = xsize;
    canvas.height = ysize;
}
function handleWebWorker(e){
    var workerData = e.data;
    // console.log('Message received from worker');
    // console.log(workerData.type);
    if (workerData.type === "ready"){
        //initialize web worker with default RLE
        myWorker.postMessage({
            type: "set_rle",
            data: RLE_STR,
        });
        //initialize web worker garbage select value
        handleGarbageSelect();
        //start calc-render loop
        renderLoop()
    }
    if (workerData.type === "serialized_tree"){
        tree.free();
        tree = TreeDataWrapper.deserialize_treerepr(workerData.data);
        workerhashcount = workerData.hash_count;
        render();
        renderLoop();
    }
}
function handleGarbageSelect(e){
    myWorker.postMessage({
        type: "set_garbage_limit",
        amount: garbageLimit(),
    })
}
var is_mouse_down = false;
var xcursor = 0;
var ycursor = 0;
function handle_mousedown(event){
    // console.log("start")
    // console.log(event)
    is_mouse_down = true;
    if(event.changedTouches){
        event = event.changedTouches[0];
    }
    xcursor = event.clientX;
    ycursor = event.clientY;
    event.stopPropagation();
}
function handle_mouseup(event){
    is_mouse_down = false;
}
function handle_mousemose(event){
    if (is_mouse_down){
        if(event.changedTouches){
            event = event.changedTouches[0];
        }
        // console.log(event)
        var deltax = (event.clientX - xcursor)
        var deltay = (event.clientY - ycursor)
        xstart -= deltax*zoomScale();
        ystart -= deltay*zoomScale();
        xcursor += deltax;
        ycursor += deltay;
        render();
        event.stopPropagation();
    }
}
function fetch_data(url, callback){
    // 1. Create a new XMLHttpRequest object
    let xhr = new XMLHttpRequest();

    // 2. Configure it: GET-request for the URL /article/.../load
    xhr.open('GET', url);

    // 3. Send the request over the network
    xhr.send();

    // 4. This will be called after the response is received
    xhr.onload = function() {
    if (xhr.status != 200) { // analyze HTTP status of the response
        alert(`Error ${xhr.status}: ${xhr.statusText}`); // e.g. 404: Not Found
    } else { // show the result
        callback(xhr.response)
    }
    };
}
function onSelectChange(event){
    const selected_url = examplesSelect.options[examplesSelect.selectedIndex].value;
    console.log("loading: " + selected_url)
    fetch_data(selected_url, (result)=>{
        handleRleUpdate(result);
    })
}
function handlePlayPause(event){
    if (is_paused){
        play_pause.innerHTML = '<i class="material-icons">pause</i>';
        is_paused = false;
    }
    else{
        play_pause.innerHTML = '<i class="material-icons">play_arrow</i>';
        is_paused = true;
    }
}
play_pause.onclick = handlePlayPause;

examplesSelect.addEventListener('change',onSelectChange);
brightnessSelect.addEventListener('change',render);
resetBoundingButton.addEventListener("click", resetBoundingBox, false);
downloadButton.addEventListener("click", downloadRLE, false);
garbageSelect.addEventListener('change', handleGarbageSelect);
inputFileLoader.addEventListener("change", handleFileUpload, false);
canvas.addEventListener("wheel", handle_wheel, false);
window.addEventListener('resize', onWindowResize);
canvas.addEventListener('mousemove', handle_mousemose);
window.addEventListener('mouseup', handle_mouseup);
window.addEventListener('mousedown', handle_mousedown);
window.addEventListener('touchmove', handle_mousemose);
window.addEventListener('touchend', handle_mouseup);
window.addEventListener('touchstart', handle_mousedown);

myWorker.onmessage = handleWebWorker

canvas.width = xsize;
canvas.height = ysize;

set_panic_hook_js();
clearCanvas()
resetBoundingBox()

//request first render
actualRender()
requestAnimationFrame(actualRender);

}
run()