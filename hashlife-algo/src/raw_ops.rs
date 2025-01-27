
// const NULL_VALUE: QuadTreeValue = QuadTreeValue{
//     lt: NULL_KEY,
//     rt: NULL_KEY,
//     lb: NULL_KEY,
//     rb: NULL_KEY,
// };
// const NULL_NODE: QuadTreeNode = QuadTreeNode{
//     v: NULL_VALUE,
//     forward_key: NULL_KEY,
//     forward_steps: 0,
//     set_count: 0xcccccccc,
//     age: 0xffffffff,
// };

pub fn node_is_raw(x:u128)->bool{
    // is the base, raw data if the top 64 bits are all 0
    // note that this should result in a collision with a real hash
    // with 1/2^64 probability
    (x >> 64) == 0
}
fn calc_result_bitsize(sums:u64, orig_vals:u64)->u64{
    //can support either 8 bit or 4 bit packing
    let mask = 0x1111111111111111 as u64;
    let bit1set = sums;
    let bit2set = sums >> 1;
    let bit4set = sums >> 2;
    let ge3 = bit1set & bit2set;
    let eq4 = bit4set & !bit1set & !bit2set;
    let eq3 = ge3 & !bit4set;
    let res = ((eq4&orig_vals) | eq3) & mask;
    res
}
fn sum_row(row: u64)->u64{
    row + (row<<4) + (row>>4)
}
fn step_forward_automata_16x16(prevmap: &[u64], nextmap: &mut[u64], step_num: usize){
    //masking by this row makes sure that extra bits on end don't get set (not technically inaccurate, just confusing)
    debug_assert!(step_num < 4);
    let rowmask = 0x0111111111111110 as u64;
    let mut s1 = sum_row(prevmap[step_num]);
    let mut s2 = sum_row(prevmap[step_num+1]);
    let mut csum = s1 + s2;
    for y in (1+step_num)..(16-1-step_num){
        let s3 = sum_row(prevmap[y+1]);
        csum += s3;
        let row_result = calc_result_bitsize(csum,prevmap[y]);
        nextmap[y] = row_result & rowmask;
        csum -= s1;
        s1 = s2;
        s2 = s3;
    }
}

const fn bits_to_4bit(x:u16)->u64{
    let q16 = x as u64;
    let q8 = (q16 | (q16 << 24)) & 0x000000ff000000ff;
    let q4 = (q8 | (q8 << 12))   & 0x000f000f000f000f;
    let q2 = (q4 | (q4 << 6))   &  0x0303030303030303;
    let q1 = (q2 | (q2 << 3))   &  0x1111111111111111;
    q1
}
const MAP_SIZE:usize = 1<<16;
const fn generate_bit_to_4bit_mapping()->[u64;MAP_SIZE]{
    let mut cached_map = [0 as u64;MAP_SIZE];
    let mut i = 0;
    // using a while loop because `for`, `map`, etc,
    // do not work in constant function as of 2021 stable release
    while i < MAP_SIZE{
        cached_map[i] = bits_to_4bit(i as u16);
        i += 1;
    }
    cached_map
}
const BIT4_MAPPING:[u64;MAP_SIZE] = generate_bit_to_4bit_mapping();
fn to_4bit(x: u16) -> u64{
    BIT4_MAPPING[x as usize]
}
fn pack_4bit_to_bits(x:u32)->u8{
    let g1 = x & 0x11111111;
    let g2 = ((g1 >> 3) | g1) & 0x03030303;
    let g4 = ((g2 >> 6) | g2) & 0x000f000f;
    let g8 = ((g4 >> 12) | g4) & 0x0000000ff;
    g8 as u8
}
fn unpack_to_bit4(d: [u128;4]) -> [u64;16]{
    let dataarr = d.map(|x|x as u64);
    let dataarr_bytes = unsafe{std::mem::transmute::<[u64; 4], [u8;32]>(dataarr)};
    let mut blocked_bytes = [0 as u64;16];
    for y in 0..16 {
        let b = (y/8)*8;
        blocked_bytes[y] = to_4bit(dataarr_bytes[y+b] as u16 + ((dataarr_bytes[y+b+8] as u16) << 8));
    }
    blocked_bytes
}
fn get_inner_8x8(data: &[u64])->[u32;8]{
    let mut inner_words = [0 as u32; 8];
    for y in 0..8{
        inner_words[y] = (data[y+4] >> 16) as u32;
    }
    inner_words
}
fn pack_finished_bit4(data: [u32;8]) -> u64{
    let packed_inner_blocks = data.map(pack_4bit_to_bits);
    unsafe{std::mem::transmute::<[u8; 8], u64>(packed_inner_blocks)}
}
pub fn step_forward_raw(d: [u128;4], n_steps: u64) -> u128{
    assert!(n_steps <= 4);
    let mut data1 = unpack_to_bit4(d);
    let mut data2 = [0 as u64;16];
    for step in 0..n_steps{
        if step%2 == 0{
            step_forward_automata_16x16(&data1[..], &mut data2[..], step as usize);
        }
        else{
            step_forward_automata_16x16(&data2[..], &mut data1[..], step as usize);
        }
    }
    let final_data =  if n_steps%2 == 0 {&data1[..]} else {&data2[..]};
     pack_finished_bit4(get_inner_8x8(final_data)) as u128
}
pub fn transpose_quad(im:&[u128;16])->[u128;16]{
    //transpose 2x2 quads (each of which are 2x2) into a 4x4 grid
    [
        im[0], im[1], im[4], im[5],
        im[2], im[3], im[6], im[7],
        im[8], im[9], im[12],im[13],
        im[10],im[11],im[14],im[15],
    ]
}

pub fn is_on_4x4_border(i: usize)->bool{
    [
        0, 1, 2, 3,
        4,       7,
        8,       11,
        12,13,14,15,
    ].iter().any(|x|*x == i)
}
pub fn slice(in_map:&[u128;16], x: usize, y: usize)->[u128;4]{
    [
        in_map[(0+y)*4+0+x], in_map[(0+y)*4+1+x],
        in_map[(1+y)*4+0+x], in_map[(1+y)*4+1+x],
    ]
}
fn rep_bytes(v: u8)->u64{
    let v1 = v as u64;
    let v2 = v1 | (v1 << 32);
    let v4 = v2 | (v2 << 16);
    let v8 = v4 | (v4 << 8);
    v8
}
fn get_gray_mask(d: i64)-> u64{
    let nds = 1<<(d+2);
    let xmask = rep_bytes((1<<nds) - 1);
    let ymask = ((1 as u64)<<(nds*8))- 1;
    let mask = xmask & ymask;
    mask
}
pub fn get_subchunk(v: u64, d: i64, x: u8, y: u8)->u64{
    let nds = 1<<(d+2);
    let xshift = v >> (nds * x);
    let yshift = xshift >> (nds*8*y);
    get_gray_mask(d) & yshift 
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_step_forward_16x16() {
        let value_map:[u64;16] = [
            0x1001110110011101,
            0x1011110110111101,
            0x1000110110001101,
            0x1101110000000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1001110100000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1011010011001110,
            0x1011110111001111,
            0x1000110111001101,
            0x1101110011001101,
        ];
        let expected_out:[u64;16] = [
            0x0000000000000000,
            0x0010000000100000,
            0x0000000111000100,
            0x0111000000000000,
            0x0000011000000000,
            0x0000011000000000,
            0x0000011000000000,
            0x0000010000000000,
            0x0000010000000000,
            0x0000010000000000,
            0x0000011000000000,
            0x0000011000000100,
            0x0000010101001000,
            0x0010000000110000,
            0x0000000000110000,
            0x0000000000000000,
        ];
        let mut out_value_map = [0 as u64; 16];
        step_forward_automata_16x16(&value_map, &mut out_value_map, 0);
        assert_eq!(out_value_map, expected_out);

    }
    #[test]
    fn test_step_forward_glider() {
        let value_map:[u64;16] = [
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000010000000,
            0x0000000001000000,
            0x0000000111000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
        ];
        let expected_out:[u64;16] = [
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000101000000,
            0x0000000011000000,
            0x0000000010000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
            0x0000000000000000,
        ];
        let mut out_value_map = [0 as u64; 16];
        step_forward_automata_16x16(&value_map, &mut out_value_map, 0);
        assert_eq!(out_value_map, expected_out);

    }
    #[test]
    fn test_get_inner_8() {
        let map16x16:[u64;16] = [
            0x1001110110011101,
            0x1011110110111101,
            0x1000110110001101,
            0x1101110000000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1000110111001101,
            0x1011010000000000,
            0x1011010000000000,
            0x1011010011001110,
            0x1001110100000000,
            0x1001010000000000,
            0x1011010000000000,
            0x1011010000000000,
            0x1011110111001111,
            0x1101110011001101,
        ];
        let expecteded_8x8:[u32;8] = [
            0x01000000,
            0x01000000,
            0x11011100,
            0x01000000,
            0x01000000,
            0x01001100,
            0x11010000,
            0x01000000,
        ];
        assert_eq!(get_inner_8x8(&map16x16[..]), expecteded_8x8);
    }
    #[test]
    fn test_packbits(){
        let maps:[[u32;8];4] = [
            [
                0x01000000,
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01001100,
                0x11010000,
                0x01000000,
            ],
            [
                0x11010000,
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01000000,
                0x01001100,
                0x01000000,
            ],
            [
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01000000,
                0x01001100,
                0x11010000,
                0x01000000,
            ],
            [
                0x01000000,
                0x01000000,
                0x11011100,
                0x01000000,
                0x01000000,
                0x01001100,
                0x01000000,
                0x11010000,
            ]
        ];
        let expected_map: [u64;16] = [
            0x1101000001000000,
            0x0100000001000000,
            0x1101110011011100,
            0x0100000001000000,
            0x0100000001000000,
            0x0100000001001100,
            0x0100110011010000,
            0x0100000001000000,
            0x0100000001000000,
            0x0100000011011100,
            0x1101110001000000,
            0x0100000001000000,
            0x0100000001000000,
            0x0100110001001100,
            0x0100000011010000,
            0x1101000001000000,
        ];
        let bits64 = maps.map(|x|pack_finished_bit4(x));
        let expectedbits64: [u64; 4] = [
            0x40D04C4040DC4040,
            0x404C404040DC40D0,
            0x40D04C404040DC40,
            0xD0404C4040DC4040,
        ];

        assert_eq!(bits64, expectedbits64);
        let value = bits64.map(|x| x as u128);
        let unpacked = unpack_to_bit4(value);
        assert_eq!(unpacked, expected_map);
    }
    #[test]
    fn test_cmprison(){
        let sumval: u64 = 0x3412750434127504;
        let curval: u64 = 0x0100000101000001;
        let expected: u64 = 0x1100000111000001;
        let actual = calc_result_bitsize(sumval, curval);
        assert_eq!(actual, expected);
    }
    #[test]
    fn test_bit4_op(){
        assert_eq!(to_4bit(0xa7),0x10100111);
    }
    #[test]
    fn test_bit4_op_back(){
        assert_eq!(pack_4bit_to_bits(0x10100111),0xa7);
    }
    #[test]
    fn test_transpose_quad(){
        let orig_arr: [u128;16] =  [
            1, 2, 3, 4,
            5, 6, 7, 8,
            9, 10,11,12,
            13,14,15,16
        ];
        let expected: [u128;16] =[
            1, 2, 5, 6,
            3, 4, 7, 8,
            9, 10,13,14,
            11,12,15,16
        ];
        assert_eq!(transpose_quad(&orig_arr), expected);
    }
    #[test]
    fn test_slice(){
        let orig_arr: [u128;16] = [
            1, 2, 5, 6,
            3, 4, 7, 8,
            9, 10,13,14,
            11,12,15,16
        ];
        let expected: [u128;4] = [
            4, 7,
            10, 13,
        ];
        assert_eq!(slice(&orig_arr,1,1), expected);
    }
    #[test]
    fn test_rep_bytes(){
        assert_eq!(rep_bytes(3),0x0303030303030303 as u64);
    }
    #[test]
    fn test_get_graymask(){
        assert_eq!(get_gray_mask(0),  0x000000000f0f0f0f);
        assert_eq!(get_gray_mask(-1), 0x0000000000000303);
        assert_eq!(get_gray_mask(-2), 0x0000000000000001);
    }
    #[test]
    fn test_get_chunk(){
        assert_eq!(get_subchunk(0x5432109876543210, 0, 0, 0),0x06040200);
        assert_eq!(get_subchunk(0x5432109876543210, 0, 0, 1),0x04020008);
        assert_eq!(get_subchunk(0x5432109876543210, 0, 1, 0),0x07050301);
        assert_eq!(get_subchunk(0x5432109876543210, 0, 1, 1),0x05030109);
        assert_eq!(get_subchunk(0x06040201, -1, 0, 0),0x0201);
        assert_eq!(get_subchunk(0x06040201, -1, 0, 1),0x0200);
        assert_eq!(get_subchunk(0x06040201, -1, 1, 0),0x0000);
        assert_eq!(get_subchunk(0x06040201, -1, 1, 1),0x0101);
    }


}