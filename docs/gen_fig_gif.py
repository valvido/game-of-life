from PIL import Image, ImageDraw
import PIL

IMG_SZ = 81
back = (50,0,200,int(255*0.2))
redsquare = (255,0,130,int(255*0.3))
subseg = (0,0,200,int(255*0.3))
subred = (255,0,130,int(255*0.3))

def draw_board(pos, step):
    x,y = pos
    # draw squares
    astep = 1-step
    base = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,255))
    background = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,0))
    bkdraw = ImageDraw.Draw(background)
    bkdraw.rectangle(((step*10, step*10), (80-step*10, 80-step*10)), fill=back)
    bkdraw.rectangle(((10+10*step, 10+10*step), (70-step*10, 70-step*10)), fill=redsquare)
    myrect = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,0))
    rctdraw = ImageDraw.Draw(myrect)
    rctdraw.rectangle(((x*10, y*10), (x*10+40, y*10+40)), fill=subseg)
    rctdraw.rectangle((((x+1)*10, (y+1)*10), ((x+1)*10+20, (y+1)*10+20)), fill=subred)

    grid = Image.new('RGBA', (IMG_SZ, IMG_SZ), (255,255,255,0))
    grddraw = ImageDraw.Draw(grid)
    for i in range(9):
        y = i * 10
        grddraw.line(((0, y), (80, y)), fill='black', width=1)
        x = i * 10
        grddraw.line(((x, 0), (x, 80)), fill='black', width=1)

    base = Image.alpha_composite(base, background)
    base = Image.alpha_composite(base, myrect)
    base = Image.alpha_composite(base, grid)
    source_img = base.resize((base.size[0]*8,base.size[1]*8),Image.Resampling.NEAREST)
    return source_img#.convert('RGB')

def generate_jmages():
    for y in range(0,6,2):
        for x in range(0,6,2):
            yield draw_board((x,y), 0)
    for y in range(1,5,2):
        for x in range(1,5,2):
            yield draw_board((x,y), 1)

def dup_images_for_slowness(img_gen):
    for img in img_gen:
        for i in range(65):
            yield img

def save_gif():
    img_generator = dup_images_for_slowness(generate_jmages())
    img1 = next(img_generator)

    img1.save('docs/checkerboard.gif',
                save_all=True, append_images=img_generator, optimize=True, duration=6, loop=0)

save_gif()