import os
import shutil
from pathlib import Path

# build with 
# wasm-pack build --release --target web
# if os.path.exists("build"):
#     shutil.rmtree("build")
# os.mkdir("build")

shutil.copy("../wasm-hashlife/pkg/wasm_hashlife_bg.wasm","wasm_hashlife_bg.wasm");
shutil.copy("../wasm-hashlife/pkg/wasm_hashlife.js","wasm_hashlife_bg.js");

options = []
for subpath in Path("examples").iterdir():
    options.append(f'<option value="{subpath}">{os.path.basename(subpath)}</option>')
    # shutil.copy(subpath, Path("build") / os.path.basename(subpath))

options_str = "\n    ".join(options)
select = f'<select name="examples" id="examples-select">\n    {options_str}\n</select>\n'
print(select)