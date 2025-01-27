"""
To run this file, first install svgwrite:

`pip install svgwrite`

Then run the script

`python gen_figs.py`

Documentation for the library can be found at:
https://svgwrite.readthedocs.io/en/latest/
"""

import sys
import math
from pathlib import Path

try:
    import svgwrite
except ImportError:
    sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import svgwrite
if svgwrite.version < (1,0,1):
    print("This script requires svgwrite 1.0.1 or newer for internal stylesheets.")
    sys.exit()

BOARD_WIDTH = "10cm"
BOARD_HEIGHT = "10cm"
BOARD_SIZE = (BOARD_WIDTH, BOARD_HEIGHT)
CLASS_CSS_STYLES = """
    .background { fill: white; }
    .line { stroke: black; stroke-width: .1mm; }
    .background { fill: white; }
    .back { fill: rgba(50,0,200,0.2); }
    .redsquare { fill: rgba(255,0,0,0.3); }
    .subseg { fill: rgba(0,0,200,0.3); }
    .subred { fill: rgba(255,0,0,0.3); }
"""

def draw_board(dwg, group, pos, step):
    # setup element groups
    # redsquare = dwg.add(dwg.g(class_="redsquare"))
    # background = dwg.add(dwg.g(class_="back"))
    # subseg = dwg.add(dwg.g(class_="subseg"))
    # subred = dwg.add(dwg.g(class_="subred"))
    # group = dwg.add(dwg.g())

    x,y = pos
    # draw squares
    astep = 1-step
    group.add(dwg.rect(size=('100%','100%'), class_='background'))
    group.add(dwg.rect(insert=(10+10*step, 10+10*step), size=(60-step*20, 60-step*20),class_="redsquare"))
    group.add(dwg.rect(insert=(step*10, step*10), size=(80-step*20, 80-step*20),class_="back"))
    group.add(dwg.rect(insert=(x*10, y*10), size=(40, 40),class_="subseg"))
    group.add(dwg.rect(insert=((x+1)*10, (y+1)*10), size=(20, 20),class_="subred"))
    # white_squares.add(dwg.rect(insert=(0, 0), size=(100, 100)))

    # draw lines
    for i in range(9):
        y = i * 10
        group.add(dwg.line(start=(0, y), end=(80, y),class_="line"))
        x = i * 10
        group.add(dwg.line(start=(x, 0), end=(x, 80),class_="line"))


def make_css():
    n_svgs = 13
    delay = 0.5
    perc_shown =  ((100/n_svgs)) # 8
    css_data = CLASS_CSS_STYLES + f'''
    @keyframes cf4FadeInOut {{
    0% {{
        opacity:1;
    }}
    {math.ceil(perc_shown)}% {{
        opacity:1;
    }}
    {math.ceil(perc_shown)+1}% {{
        opacity:0;
    }}
    99.9999% {{
        opacity:0;
    }}
    100% {{
        opacity:1;
    }}
    }}

    #cf4a {{
        position:relative;
        height:10cm;
        width:10cm;
        margin:0 auto;
    }}
    #cf4a g {{
        position:absolute;
        left:0;
    }}

    #cf4a g{{
        animation-name: cf4FadeInOut;
        animation-timing-function: ease-in-out;
        animation-iteration-count: infinite;
        animation-duration: {n_svgs*delay}s;
    }}
    '''
    for i in range(n_svgs):
        css_data += f'''
        #g{i} {{
        animation-delay: {delay*i}s;
        }}
        '''
    return css_data

def main():
    dwg = svgwrite.Drawing(f'docs/checkerboard.svg', size=BOARD_SIZE)
    dwg.viewbox(0, 0, 80, 80)
    # checkerboard has a size of 10cm x 10cm;
    # defining a viewbox with the size of 80x80 means, that a length of 1
    # is 10cm/80 == 0.125cm (which is for now the famous USER UNIT)
    # but I don't have to care about it, I just draw 8x8 squares, each 10x10 USER-UNITS

    # always use css for styling
    dwg.defs.add(dwg.style(make_css()))

    # set background

    grouper = dwg.add(dwg.g(id="cf4a"))
    gsteps = []

    idx = 0
    for y in range(0,6,2):
        for x in range(0,6,2):
            gstep = (dwg.g(id=f"g{idx}"))
            draw_board(dwg,gstep,(x,y), 0)
            gsteps.append(gstep)
            idx += 1
    for y in range(1,5,2):
        for x in range(1,5,2):
            gstep = (dwg.g(id=f"g{idx}"))
            draw_board(dwg,gstep,(x,y), 1)
            gsteps.append(gstep)
            idx += 1
    for gstep in reversed(gsteps):
        grouper.add(gstep)
    dwg.save()



if __name__ == "__main__":
    main()