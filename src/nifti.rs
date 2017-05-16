//Definitions of the nifti datatypes
use std::error::Error;
use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

pub type NIfTI1Data = Vec<Vec<Vec<Vec<f32>>>>;

#[repr(C)]
pub struct NIfTI1Header{
    pub sizeof_hdr: i32,           //0   Size of the header. Must be 348 (bytes).
    pub data_type: [u8; 10],       //4   Not used; compatibility with analyze.
    pub db_name: [u8; 18],         //14  Not used; compatibility with analyze.
    pub extents: i32,              //32  Not used; compatibility with analyze.
    pub session_error: i16,        //36  Not used; compatibility with analyze.
    pub regular: u8,               //38  Not used; compatibility with analyze.
    pub dim_info: u8,              //39  Encoding directions (phase, frequency, slice).
    pub dim: [u16; 8],             //40  Data array dimensions.
    pub intent_p1: f32,            //56  1st intent parameter.
    pub intent_p2: f32,            //60  2nd intent parameter.
    pub intent_p3: f32,            //64  3rd intent parameter.
    pub intent_code: i16,          //68  nifti intent.
    pub datatype: i16,             //70  Data type.
    pub bitpix: i16,               //72  Number of bits per voxel.
    pub slice_start: i16,          //74  First slice index.
    pub pixdim: [f32; 8],          //76  Grid spacings (unit per dimension).
    pub vox_offset: f32,           //108 Offset into a .nii file.
    pub scl_slope: f32,            //112 Data scaling, slope.
    pub scl_inter: f32,            //116 Data scaling, offset.
    pub slice_end: i16,            //120 Last slice index.
    pub slice_code: u8,            //122 Slice timing order.
    pub xyzt_units: u8,            //123 Units of pixdim[1..4].
    pub cal_max: f32,              //124 Maximum display intensity.
    pub cal_min: f32,              //128 Minimum display intensity.
    pub slice_duration: f32,       //132 Time for one slice.
    pub toffset: f32,              //136 Time axis shift.
    pub glmax: i32,                //140 Not used; compatibility with analyze.
    pub glmin: i32,                //144 Not used; compatibility with analyze.
    pub descrip: [u8; 80],         //148 Any text.
    pub aux_file: [u8; 24],        //228 Auxiliary filename.
    pub qform_code: i16,           //252 Use the quaternion fields.
    pub sform_code: i16,           //254 Use of the affine fields.
    pub quatern_b: f32,            //256 Quaternion b parameter.
    pub quatern_c: f32,            //260 Quaternion c parameter.
    pub quatern_d: f32,            //264 Quaternion d parameter.
    pub qoffset_x: f32,            //268 Quaternion x shift.
    pub qoffset_y: f32,            //272 Quaternion y shift.
    pub qoffset_z: f32,            //276 Quaternion z shift.
    pub srow_x: [f32; 4],          //280 1st row affine transform
    pub srow_y: [f32; 4],          //296 2nd row affine transform
    pub srow_z: [f32; 4],          //312 3rd row affine transform
    pub intent_name: [u8; 16],     //328 Name or meaning of the data.
    pub magic: [u8; 4],            //344 Magic string.
}

pub trait New {
    fn init(&NIfTI1Header) -> NIfTI1Data;
}

impl New for NIfTI1Data {
    fn init(header: &NIfTI1Header) -> NIfTI1Data {

        let mut data: NIfTI1Data = Vec::new();
        
        for t in 0..header.dim[4]{
            data.push(Vec::new());
            for z in 0..header.dim[3]{
                data[t as usize].push(Vec::new());
                for y in 0..header.dim[2]{
                    data[t as usize][z as usize].push(Vec::new());
                    for _ in 0..header.dim[1]{
                        data[t as usize][z as usize][y as usize].push(0.0);
                    }
                }
            }
        }
        return data;
    }
}

pub fn read(file_name: &str) -> (NIfTI1Header, NIfTI1Data){

    //Open file
    let path = Path::new(file_name);
    let display = path.display();
    let mut file = match File::open(&path){
        Err(why) => panic!("Could not open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    //Read header
    let mut in_header: [u8; 348] = unsafe{ mem::zeroed() } ;
    let header_result = match file.read(&mut in_header){
        Err(why) => panic!("Could not read header of {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };
    //Check read header size
    if header_result != (mem::size_of::<NIfTI1Header>()) {
        panic!("Wrong header size!");
    }

    //Convert read header into NIfTI1Header
    let header: NIfTI1Header = unsafe {
        mem::transmute(in_header)
    };
    //Check integrety
    if header.sizeof_hdr != 348 {
        panic!("Wrong header size!");
    }
    
    //Read data
    let mut in_data: Vec<u8> = Vec::new();
    let data_result = match file.read_to_end(&mut in_data){
        Err(why) => panic!("Could not read data of {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };
    
    //Convert read data into Vec<f32>
    let mut tmp_data: Vec<f32> = Vec::new();
    let mut tmp: [u8; 4] = [0,0,0,0];
    let mut counter = 0;

    //Cast every 4 bytes to a f32
    for elem in in_data{
        tmp[counter % 4] = elem;
        
        //We got four bytes
        if counter % 4 == 3 {
            let f: f32 = unsafe {
                mem::transmute(tmp)
            };
            tmp_data.push(f);
        }
        counter += 1;
    }

    counter = 1;
    let mut data: NIfTI1Data = Vec::new();

    //Convert data into NIfTI1Data
    for t in 0..header.dim[4]{
        data.push(Vec::new());
        for z in 0..header.dim[3]{
            data[t as usize].push(Vec::new());
            for y in 0..header.dim[2]{
                data[t as usize][z as usize].push(Vec::new());
                for _ in 0..header.dim[1]{
                    data[t as usize][z as usize][y as usize].push(tmp_data[counter]);
                    counter+=1;
                }
            }
        }
    }
    
    //Check read data size
    if data_result/4 != tmp_data.len() {
        panic!("Something went wrong while converting the data. Read data: {}, converted data: {}",
               data_result-4, tmp_data.len());
    }
    
    return (header, data);
}

pub fn write(header: NIfTI1Header, data: NIfTI1Data, file_name: &str){

    //Open file
    let path = Path::new(file_name);
    let display = path.display();
    let mut file = match File::create(&path){
        Err(why) => panic!("Could not create {}: {}",
                           display, Error::description(&why)),
        Ok(file) => file,
    };

    //Write header
    let out_header: [u8; 348] = unsafe {
        mem::transmute(header)
    };
    match file.write_all(&out_header){
        Err(why) => panic!("Could not write header of {}: {}",
                           display, Error::description(&why)),
        Ok(file) => file,
    };

    //Write a leading 0
    let mut tmp: [u8; 4] = unsafe{
        mem::transmute(0.0f32)
    };
    match file.write_all(&tmp){
        Err(why) => panic!("Could not read data of {}: {}",
                           display, Error::description(&why)),
        Ok(file) => file,
    };          

    //Write every entry in data
    for t in data.iter(){
        for z in t{
            for y in z{
                for x in y{
                    tmp = unsafe{
                        mem::transmute(*x)
                    };
                    match file.write_all(&tmp){
                        Err(why) => panic!("Could not read data of {}: {}",
                                           display, Error::description(&why)),
                        Ok(file) => file,
                    };          
                }
            }
        }
    }
    //End write data
}
