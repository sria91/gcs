use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::str::FromStr;

use clap::Parser;
use diesel::{r2d2::ConnectionManager, SqliteConnection};
use dotenv::dotenv;
use uuid::Uuid;

use gcs::*;

#[derive(Debug, Parser)]
#[clap(version, author, about)]
struct Args {
  #[clap(short, long, help = "Specifies the input text file.")]
  #[clap(parse(from_os_str))]
  input: Option<std::path::PathBuf>,
  #[clap(short, long, help = "Specifies the output text file.")]
  #[clap(parse(from_os_str))]
  output: Option<std::path::PathBuf>,
  #[clap(long, help = "List all the plateaus in the database.")]
  list_plateaus: bool,
  #[clap(long, value_name = "PLATEAU_ID", help = "Lists the rovers from the specified plateau id.")]
  list_rovers: Option<Uuid>,
  #[clap(short, long, value_name = "PLATEAU_ID", help = "Loads the specified plateau from id.")]
  plateau: Option<Uuid>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = Args::parse();

  // Read environment variables from .env file.
  dotenv().ok();

  // Initialize the database connection pool.
  let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
  let manager = ConnectionManager::<SqliteConnection>::new(database_url);
  let pool = r2d2::Pool::builder().build(manager).expect("Failed to create pool");

  if args.list_plateaus {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    match plateau::list_plateaus(50, &conn) {
      Err(error) => {
        eprint!("Failed to read plateaus from database: ");
        return Err(Box::new(error));
      }
      Ok(plateaus) => {
        println!("{}", plateaus);
        return Ok(());
      }
    }
  }

  if let Some(plateau_id) = args.list_rovers {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    match plateau::list_rovers(plateau_id, 50, &conn) {
      Err(error) => {
        eprint!("Failed to read rovers from database: ");
        return Err(Box::new(error));
      }
      Ok(rovers) => {
        println!("{}", rovers);
        return Ok(());
      }
    }
  }

  // Initialize the input.
  let input: Box<dyn BufRead> = match &args.input {
    Some(input_path) => match File::open(&input_path) {
      Ok(content) => Box::new(BufReader::new(content)),
      Err(error) => {
        eprint!("Failed to read file {:?}: ", input_path);
        return Err(Box::new(error));
      }
    },
    None => Box::new(BufReader::new(io::stdin())),
  };
  let mut numbered_lines_input = input.lines().enumerate().peekable();

  // Read the plateau bounds and create the plateau.
  let plateau = match args.plateau {
    Some(plateau_id) => {
      let conn = pool.get().expect(CONNECTION_POOL_ERROR);
      match plateau::get_plateau(plateau_id, &conn) {
        Ok(plateau) => plateau,
        Err(error) => {
          return Err(Box::new(error));
        }
      }
    }
    None => {
      if args.input.is_none() {
        println!("Enter the plateau bounds:");
      }
      match numbered_lines_input.next() {
        Some((n, result)) => match result {
          Ok(line) if !line.is_empty() => match Plateau::from_str(&line) {
            Ok(plateau) => {
              if args.input.is_none() {
                println!("Created plateau with id '{}' and bounds '{}'.", plateau.id(), plateau);
              }
              plateau
            }
            Err(error) => {
              eprint!("Error on line {}: \"{}\": ", n + 1, &line);
              return Err(Box::new(error));
            }
          },
          Ok(_) => return Ok(()),
          Err(error) => {
            return Err(Box::new(error));
          }
        },
        None => {
          return Err(Box::new(Error::EmptyFile));
        }
      }
    }
  };
  dprintln!("plateau = {:?}", plateau);
  {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    match plateau::create_plateau(plateau.clone(), &conn) {
      Err(error) => {
        eprint!("Failed to create plateau in database: ");
        return Err(Box::new(error));
      }
      _ => (),
    }
  }

  let mut rover_number: i64 = 0;
  let mut rovers = Vec::new();
  loop {
    if args.input.is_none() {
      println!("Enter the intial pose for rover {}:", rover_number + 1);
    }

    if numbered_lines_input.peek().is_none() {
      break;
    }

    let mut rover = match args.plateau {
      Some(plateau_id) => {
        let conn = pool.get().expect(CONNECTION_POOL_ERROR);
        match plateau::get_rover_n(rover_number, plateau_id, &conn) {
          Ok(rover) => rover,
          Err(_) => return Ok(()),
        }
      }
      None => {
        // Get initial pose of the rover and create the rover.
        let initial_pose = match numbered_lines_input.next() {
          Some((n, result)) => match result {
            Ok(line) if !line.is_empty() => match Pose::from_str(&line) {
              Ok(pose) => pose,
              Err(error) => {
                eprint!("Error on line {}: \"{}\": ", n + 1, &line);
                return Err(Box::new(error));
              }
            },
            Ok(_) => return Ok(()),
            Err(error) => {
              eprintln!("Error: {}", error);
              return Err(Box::new(error));
            }
          },
          None => return Err(Box::new(Error::InitialPoseNotFound)),
        };
        let rover = Rover::new(initial_pose.x(), initial_pose.y(), initial_pose.facing());
        if args.input.is_none() {
          println!("Created rover with id '{}' and pose '{}'", rover.id(), rover.pose());
        }
        {
          let conn = pool.get().expect(CONNECTION_POOL_ERROR);
          match plateau::create_rover(Uuid::from_str(plateau.id()).unwrap(), rover.clone(), &conn) {
            Err(error) => {
              eprint!("Failed to create rover in database: ");
              return Err(Box::new(error));
            }
            _ => (),
          }
        }
        rover
      }
    };
    dprintln!("rover = {:?}", rover);

    if args.input.is_none() {
      println!("Enter the motion path for rover {}:", rover_number + 1);
    }
    // Get the path for the rover and move it accordingly.
    let motion_vector = match numbered_lines_input.next() {
      Some((n, result)) => match result {
        Ok(line) if !line.is_empty() => match Motion::parse_path(&line) {
          Ok(motion_vector) => motion_vector,
          Err(error) => {
            eprint!("Error on line {}: \"{}\": ", n + 1, &line);
            return Err(Box::new(error));
          }
        },
        Ok(_) => return Ok(()),
        Err(error) => {
          eprint!("Error: ");
          return Err(Box::new(error));
        }
      },
      None => return Err(Box::new(Error::PathNotFound)),
    };
    dprintln!("path of rover = {:?}", motion_vector);
    {
      let conn = pool.get().expect(CONNECTION_POOL_ERROR);
      rover = match plateau::move_rover(
        Uuid::from_str(plateau.id()).unwrap(),
        Uuid::from_str(rover.id()).unwrap(),
        motion_vector.clone(),
        &conn,
      ) {
        Err(error) => {
          eprint!("Failed to create rover in database: ");
          return Err(Box::new(error));
        }
        Ok(rover) => rover,
      }
    }

    if args.input.is_none() {
      print!("Rover {} is now at:", rover_number + 1);
    }
    println!("{}", rover.pose());

    rovers.push(rover);

    match args.plateau {
      Some(plateau_id) => {
        let conn = pool.get().expect(CONNECTION_POOL_ERROR);
        if let Ok(count) = plateau::get_rovers_count(plateau_id, &conn) {
          if rover_number as usize == count {
            break;
          }
        }
      }
      None => (),
    };

    rover_number += 1;
  }

  match args.output {
    Some(output_path) => match File::create(&output_path) {
      Ok(mut file) => {
        for rover in rovers {
          match file.write_fmt(format_args!("{}\n", rover.pose())) {
            Ok(_) => (),
            Err(error) => {
              eprint!("Failed to write to output file: ",);
              return Err(Box::new(error));
            }
          };
        }
      }
      Err(error) => {
        eprint!("Failed to create output file: ",);
        return Err(Box::new(error));
      }
    },
    None => (),
  }

  Ok(())
}
