mod nifti;
mod trackvis;

extern crate getopts;

use std::collections::HashMap;
use std::env;
use std::process::exit;
use getopts::Options;
use nifti::New;

static LH_WM_LABEL: f32 = 2.0;
static RH_WM_LABEL: f32 = 41.0;

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} <trk_file> [options]", program);
    print!("{}", opts.usage(&brief));
}

//Return probability for a given label in a label list
fn label_prob(labels: &Vec<i32>, label: i32) -> f32 {
    let mut label_count = 0.0;

    for i in 0..labels.len(){
        if labels[i] == label {
            label_count += 1.0;
        }
    }

    return label_count / labels.len() as f32;
}

fn rel_dist(x: f32, y: f32, z: f32) -> f32 {
    return (x * x + y * y + z * z).sqrt();
}

fn main() {
    /*
     Beginning of parsing command line arguments/options
    */
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("n", "nifti", "path to nifti image that represents the cortex parcellation [required]", "FILE");
    opts.optopt("o", "output", "path to the output file [optional]", "FILE");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]){
        Err(f) => panic!(f.to_string()),
        Ok(m) => m,
    };

    //Help requested
    if matches.opt_present("h"){
        print_usage(&program, &opts);
        exit(1);
    }

    //Parse options
    let nifti_file: String = match matches.opt_str("n"){
        None => {
            print_usage(&program, &opts);
            exit(1);
        },
        Some(s) => s,
    };

    let output_file: String = match matches.opt_str("o"){
        None =>  String::new(),
        Some(s) => s,
    };

    //Parse argument
    let track_file = if matches.free.len() == 1{
        matches.free[0].clone()
    } else {
        print_usage(&program, &opts);
        exit(1);
    };

    println!("TrackVis Input: {}, NIfTI1 Input: {}, Output: {}",
             track_file, nifti_file, output_file);

    /*
    End of parsing command line arguments/options
    */

    //Read the mandantory data
    let (nheader, ndata) = nifti::read( &nifti_file );
    let (_, tracts) = trackvis::read( &track_file );

    //First iteration through the fibers. Build up the label lists
    let mut label_lists: HashMap<trackvis::Position, Vec<i32>> = HashMap::new();

    for tract in tracts.iter() {
        //The group is determined by the value in the segmentation file (e.g. asec+aparc)
        //at the position of the first cortex element of the fiber
        let mut label = -1.0;

        //Determine label
        for pos in tract.iter() {

            //Go along the fiber until we get the first cortex label
            //TODO: Replace the hard coded cortex check
            let group = ndata[0][pos.z as usize][pos.y as usize][pos.x as usize];
            if (group >= 1001.0 && group <= 1035.0) || (group >= 2001.0 && group <= 2035.0){
                label = group;
            }
        }

        //Add label to every voxel in tract if there was a associating cortex label
        if label > -1.0{
            for pos in tract.iter() {
                if label_lists.contains_key(pos) {
                    label_lists.get_mut(pos).unwrap().push(label as i32);
                } else {
                    label_lists.insert(trackvis::Position{x: pos.x, y: pos.y, z: pos.z}, vec![label as i32]);
                }
            }
        }
    }

    //Choose label for every voxel
    let mut final_labels: HashMap<trackvis::Position, i32> = HashMap::new();

    for (pos, labels) in label_lists.iter() {

        let mut highest_prob = 0f32;
        let mut highest_label = 0;

        //Get the number of counts in the label list
        for i in 0..labels.len(){
            //Compute probability for the current voxel
            let curr_prob = label_prob(labels, labels[i]);

            //Compute commulative probability for neighbouring voxels
            let mut neigh_prob = 0f32;
            let mut neigh_count = 0i32;
            for iz in -1..1{
                for iy in -1..1{
                    for ix in -1..1{
                        if iz == 0 && iy == 0 && ix == 0 {
                            //Skip current position
                            continue;
                        }

                        match label_lists.get(&trackvis::Position{x: pos.x + ix, y: pos.y + iy, z: pos.z + iz}) {
                            Some(l) => {
                                let np = label_prob(l, labels[i]);
                                if np > 0.0 {
                                    neigh_prob += (1.0 / rel_dist(ix as f32, iy as f32, iz as f32)) * label_prob(l, labels[i]);
                                    neigh_count += 1
                                }
                            },
                            _ => {}
                        }
                    }
                }
            }
            //Norm neighbour probability
            if neigh_count == 0 {
                neigh_prob = 0.0;
            } else {
                neigh_prob /= neigh_count as f32;
            }

            //Add the probabilities up, compare to highest label, and choose new highest label
            if curr_prob + neigh_prob > highest_prob {
                highest_prob = curr_prob + neigh_prob;
                highest_label = labels[i];
            }
        }
        if highest_label > 0 {
            final_labels.insert(trackvis::Position{x: pos.x, y: pos.y, z: pos.z}, highest_label);
        }
    }

    //Write the parcellation to a NIfTI file
    if output_file.len() > 0 {
        println!("Write output");

        //TODO: Read a list with labels of the area that we want to parcellate
        //Write the groups to nifti data
        let mut outdata = nifti::NIfTI1Data::init(&nheader);
        for (pos, label) in final_labels.iter(){
            //Check if we are in the wm
            let curr_label = ndata[0][pos.z as usize][pos.y as usize][pos.x as usize];

            //Make sure we have a label that we want to color
            if curr_label == LH_WM_LABEL || curr_label == RH_WM_LABEL ||
                curr_label == 251.0 || curr_label == 252.0 || curr_label == 253.0 ||
                curr_label == 254.0 || curr_label == 255.0 {
                    //println!("Coloring in {}", *label);
                    outdata[0][pos.z as usize][pos.y as usize][pos.x as usize] = *label as f32;
            }
        }
        //Write the output
        nifti::write(nheader, outdata, &output_file);
    }

}
