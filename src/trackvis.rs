//Definitions of the trackvis datatypes
use std::error::Error;
use std::mem;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use nifti::NIfTI1Header;

#[repr(C)]
pub struct TrackVisHeader{
    pub id_string: [u8; 6],                    //0  ID string for track file. The first 5 characters must be "TRACK".
    pub dim: [u16; 3],                         //6   Dimension of the image volume.
    pub voxel_size: [f32; 3],                  //12  Voxel size of the image volume.
    pub origin: [f32; 3],                      //24  Origin of the image volume. This field is not yet being used by TrackVis. That means the origin is always (0, 0, 0).
    pub n_scalars: u16,                        //36  Number of scalars saved at each track point (besides x, y and z coordinates).
    pub scalar_name: [[u8; 10]; 20],           //38  Name of each scalar. Can not be longer than 20 characters each. Can only store up to 10 names.
    pub n_properties: u16,                     //238 Number of properties saved at each track.
    pub property_name: [[u8; 10]; 20],         //240 Name of each property. Can not be longer than 20 characters each. Can only store up to 10 names.
    pub vox_to_ras: [[f32; 4]; 4],             //440 4x4 matrix for voxel to RAS (crs to xyz) transformation. If vox_to_ras[3][3] is 0, it means the matrix is not recorded. This field is added from version 2.
    pub reserved: [u8; 444],                   //504 Reserved space for future version.
    pub voxel_order: [u8; 4],                  //948 Storing order of the original image data. Explained at http://trackvis.org/docs/ .
    pub pad2: [u8; 4],                         //952 Paddings.
    pub image_orientation_patient: [f32; 6],   //956 Image orientation of the original image. As defined in the DICOM header.
    pub pad1: [u8; 2],                         //980 Paddings.
    pub invert_x: u8,                          //982 Inversion/rotation flags used to generate this track file. For internal use only.
    pub invert_y: u8,                          //983 As above.
    pub invert_z: u8,                          //984 As above.
    pub swap_xy: u8,                           //985 As above.
    pub swap_yz: u8,                           //986 As above.
    pub swap_zx: u8,                           //987 As above.
    pub n_count: u32,                          //988 Number of tracks stored in this track file. 0 means the number was NOT stored.
    pub version: u32,                          //992 Version number. Current version is 2.
    pub hdr_size: u32,                         //996 Size of the header. Used to determine byte swap. Should be 1000.
}

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub type Fiber = Vec<Position>;

#[allow(dead_code)]
pub fn write(nheader: &NIfTI1Header, fibers: &Vec<Fiber>, file_name: &str){

    //Open file
    let path = Path::new(file_name);
    let display = path.display();
    let mut file = match File::create(&path){
        Err(why) => panic!("Could not create {}: {}",
                           display, Error::description(&why)),
        Ok(file) => file,
    };

    //Fill header
    let mut header: TrackVisHeader = unsafe { mem::zeroed() };
    header.id_string = ['T' as u8, 'R' as u8, 'A' as u8, 'C' as u8, 'K' as u8, 0 as u8];
    header.dim = [ nheader.dim[1], nheader.dim[2], nheader.dim[3] ];
    header.voxel_size = [ nheader.pixdim[1], nheader.pixdim[2], nheader.pixdim[3] ];
    header.origin = [0.0, 0.0, 0.0];
    header.n_scalars = 0;
    header.scalar_name = [[0; 10]; 20];
    header.n_properties = 0;
    header.property_name = [[0; 10]; 20];
    header.vox_to_ras[0] = unsafe{ mem::transmute(nheader.srow_x) };
    header.vox_to_ras[1] = unsafe{ mem::transmute(nheader.srow_y) };
    header.vox_to_ras[2] = unsafe{ mem::transmute(nheader.srow_z) };
    header.vox_to_ras[3] = [0.0, 0.0, 0.0, 1.0];
    header.reserved = [0u8; 444];
    header.voxel_order = ['L' as u8, 'A' as u8, 'S' as u8, 0 as u8];
    header.pad2 = header.voxel_order;
    header.image_orientation_patient = [1.0, 0.0, 0.0, 0.0, -1.0, 0.0];
    header.pad1 = [0; 2];
    header.invert_x = 0;
    header.invert_y = 0;
    header.invert_z = 0;
    header.swap_xy = 0;
    header.swap_yz = 0;
    header.swap_zx = 0;
    header.n_count = fibers.len() as u32;
    header.version = 2;
    header.hdr_size = 1000;

    //Write header
    let out_header: [u8; 1000] = unsafe {
        mem::transmute(header)
    };
    match file.write_all(&out_header){
        Err(why) => panic!("Could not write header of {}: {}",
                           display, Error::description(&why)),
        Ok(file) => file,
    };

    //Write every entry in data
    for fiber in fibers.iter(){

        //Write length of track
        let mut tmp: [u8; 4] = unsafe{
            mem::transmute(fiber.len() as u32)
        };
        match file.write_all(&tmp){
            Err(why) => panic!("Could not read data of {}: {}",
                               display, Error::description(&why)),
            Ok(file) => file,
        };

        //Write trackpoints
        //Important note: The coordinates in the track file are in mm, so you have so scale them
        for point in fiber.iter(){
            tmp = unsafe{
                mem::transmute(nheader.pixdim[1] * point.x as f32)
            };
            match file.write_all(&tmp){
                Err(why) => panic!("Could not read data of {}: {}",
                                   display, Error::description(&why)),
                Ok(file) => file,
            };
            tmp = unsafe{
                mem::transmute(nheader.pixdim[2] * point.y as f32)
            };
            match file.write_all(&tmp){
                Err(why) => panic!("Could not read data of {}: {}",
                                   display, Error::description(&why)),
                Ok(file) => file,
            };
            tmp = unsafe{
                mem::transmute(nheader.pixdim[3] * point.z as f32)
            };
            match file.write_all(&tmp){
                Err(why) => panic!("Could not read data of {}: {}",
                                   display, Error::description(&why)),
                Ok(file) => file,
            };

        }
    }
}

pub fn read (file_name: &str) -> (TrackVisHeader, Vec<Fiber>) {

    //Open file
    let path = Path::new(file_name);
    let display = path.display();
    let mut file = match File::open(&path){
        Err(why) => panic!("Could not open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    //Read header
    let mut in_header: [u8; 1000] = unsafe{ mem::zeroed() } ;
    let header_result = match file.read(&mut in_header){
        Err(why) => panic!("Could not read header of {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    //Check read header size
    if header_result != (mem::size_of::<TrackVisHeader>()) {
        panic!("Wrong header size!");
    }

    //Convert read header into NIfTI1Header
    let header: TrackVisHeader = unsafe {
        mem::transmute(in_header)
    };

    //Check integrety
    if header.hdr_size != 1000 {
        panic!("Wrong header size!");
    }

    //Read data
    let mut in_data: Vec<u8> = Vec::new();
    match file.read_to_end(&mut in_data){
        Err(why) => panic!("Could not read data of {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    let mut fibers: Vec<Fiber> = Vec::new();
    let mut tmp_data: [u8; 4] = [0,0,0,0];

    let mut data_pos = 0;

    while data_pos < in_data.len() {

        //The first quartett is the number of stored Track Points
        for i in 0..4 {
            //Add four bytes to an int
            tmp_data[i] = in_data[data_pos];
            data_pos += 1;
        }

        let num_points: u32 = unsafe {
            mem::transmute(tmp_data)
        };

        //Read the whole fiber
        let mut tmp_fiber: Fiber = Vec::new();

        for _ in 0..num_points {

            //We do for the three coordinate the same:
            //Convert four bytes to a f32
            //Assign it to the right variable in tmp_position
            //Add the read position to a fiber
            let mut xyz: [f32; 3] = [0.0, 0.0, 0.0];

            for n in 0..3 {
                for i in 0..4 {
                    tmp_data[i] = in_data[data_pos];
                    data_pos += 1;
                }
                let tmp_coord: f32 = unsafe {
                    mem::transmute(tmp_data)
                };

                //The TrackVis Coordinates are given in mm.
                //Therefore we convert the TrackVis coordinate to coordinates in the NIfTI image.
                xyz[n] = tmp_coord / header.voxel_size[n];
                xyz[n] = xyz[n].trunc();
            }
            //println!("");
            tmp_fiber.push( Position{x: xyz[0] as i32, y: xyz[1] as i32, z: xyz[2] as i32} );
        }

        //Add the read fiber to a list of fibers
        fibers.push(tmp_fiber);
    }

    //TODO: Check read data size
    /*
    if header.n_count != 0 as u32 && header.n_count != (data_result/4 - fibers.len()) as u32 {
        panic!("Something went wrong while reading the tracks. Expected {} tracks, but got {} tracks",
               data_result/4, fibers.len());
    }
     */


    return (header, fibers);

}
