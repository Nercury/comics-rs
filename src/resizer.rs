use std::path::{ Path, PathBuf };
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{ Read, Write };
use image;
use image::GenericImage;
use image::DynamicImage;
use serde_json;
use resizer_models::SizeRepr;

#[derive(Debug)]
pub enum ResizeMode {
    Fit(SizeHint),
    Fill(Size),
}

#[derive(Debug)]
pub struct SizeHint {
    pub w: Option<u32>,
    pub h: Option<u32>,
}

#[derive(Debug, Copy, Clone)]
pub struct Size {
    pub w: u32,
    pub h: u32,
}

pub struct Resizer {
    sizes: HashMap<PathBuf, Size>,
    root_path: PathBuf,
    resize_cache: PathBuf,
}

pub struct ResizeResult {
    pub path: PathBuf,
    pub relative_url: String,
    pub size: Size,
}

fn ratio(w: u32, h: u32) -> Result<f32, ()> {
    if h == 0 {
        Err(())
    } else {
        Ok((w as f32) / (h as f32))
    }
}

fn aspect_h(ratio: Result<f32, ()>, w: u32) -> Result<u32, ()> {
    match ratio {
        Ok(ratio) => Ok(((w as f32) / ratio) as u32),
        _ => Err(()),
    }
}

fn aspect_w(ratio: Result<f32, ()>, h: u32) -> Result<u32, ()> {
    match ratio {
        Ok(ratio) => Ok(((h as f32) * ratio) as u32),
        _ => Err(()),
    }
}

pub fn get_required_size(o: Size, mode: &ResizeMode) -> Result<Size, ()> {
    match *mode {
        ResizeMode::Fit(ref hint) => match *hint {
            SizeHint { w: Some(w), h: None } if w > o.w => Ok(Size { w: o.w, h: o.h }),
            SizeHint { w: None, h: Some(h) } if h > o.h => Ok(Size { w: o.w, h: o.h }),
            SizeHint { w: Some(w), h: None } => Ok(Size { w: w, h: try!(aspect_h(ratio(o.w, o.h), w)) }),
            SizeHint { w: None, h: Some(h) } => Ok(Size { w: try!(aspect_w(ratio(o.w, o.h), h)), h: h }),
            SizeHint { w: Some(w), h: Some(h) } => Ok(Size { w: w, h: h }),
            SizeHint { w: None, h: None } => Ok(Size { w: o.w, h: o.h }),
        },
        _ => unimplemented!(),
    }
}

impl Resizer {
    pub fn new(root_path: &Path, resize_cache: &Path) -> Resizer {
        Resizer {
            sizes: HashMap::new(),
            root_path: root_path.into(),
            resize_cache: resize_cache.into(),
        }
    }

    fn get_memcached_size(&self, path: &Path) -> Option<Size> {
        self.sizes.get(path).cloned()
    }

    fn set_memcached_size(&mut self, path: &Path, size: Option<Size>) {
        match size {
            None => self.sizes.remove(path),
            Some(size) => self.sizes.insert(path.into(), size),
        };
    }

    fn get_filecache_path(&self, path: &Path) -> PathBuf {
        self.resize_cache.join(Path::new(path.file_name().unwrap()).with_extension("size.json"))
    }

    fn get_filecached_size<'r>(&'r self, path: &Path) -> Option<Size> {
        let mut f = match File::open(&path) {
            Ok(f) => f,
            _ => {
                return None;
            },
        };

        let mut contents = String::new();
        if let Err(e) = f.read_to_string(&mut contents) {
            println!("error reading cached size for {:?}: {:?}", path, e);
            return None;
        }

        let deserialized: SizeRepr = match serde_json::from_str(&contents) {
            Ok(contents) => contents,
            Err(e) => {
                println!("error deserializing size for {:?}: {:?}", path, e);
                return None;
            }
        };

        let res = Size { w: deserialized.w, h: deserialized.h };

        Some(res)
    }

    fn set_filecached_size(&self, path: &Path, size: Option<Size>) {
        match size {
            None => {
                let _ = fs::remove_file(path);
            },
            Some(size) => {
                let size_repr = SizeRepr {
                    w: size.w,
                    h: size.h,
                };
                if let Ok(serialized) = serde_json::to_string(&size_repr) {
                    if let Ok(mut file) = File::create(path) {
                        let _ = file.write_all(serialized.as_bytes());
                    }
                }
            }
        }
    }

    fn get_cached_size<'r>(&self, path: &'r Path, filecached_path: &'r Path)
        -> Option<(Option<DynamicImage>, Size, bool, bool)>
    {
        match self.get_memcached_size(path) {
            Some(s) => Some((None, s, false, false)),
            None => {
                match self.get_filecached_size(filecached_path) {
                    Some(s) => Some((None, s, true, false)),
                    None => {
                        let opened_image = match image::open(path) {
                            Ok(image) => image,
                            Err(e) => {
                                println!("error opening image {:?}: {:?}", path, e);
                                return None
                            },
                        };
                        let size = Size {
                            w: opened_image.width(),
                            h: opened_image.height(),
                        };

                        Some((Some(opened_image), size, true, true))
                    }
                }
            }
        }
    }

    pub fn get_resized_url<'r>(&mut self, url: &'r str, mode: ResizeMode) -> Option<ResizeResult> {
        let path = &self.root_path.join(url);
        let filecached_path = &self.get_filecache_path(path);

        let (mut image, original_size, update_memcache, update_filecache) = match self.get_cached_size(path, filecached_path) {
            Some(res) => res,
            None => return None,
        };

        if update_memcache {
            self.set_memcached_size(path, Some(original_size));
        }

        if update_filecache {
            self.set_filecached_size(filecached_path, Some(original_size));
        }

        let required_size = match get_required_size(original_size, &mode) {
            Ok(size) => size,
            _ => {
                println!("invalid image size {:?}: {:?}", path, original_size);
                return None;
            }
        };

        let size_str = [required_size.w.to_string().as_ref(), required_size.h.to_string().as_ref()].connect("x");
        let (extension, needs_resize) = if required_size.w == original_size.w && required_size.h == original_size.h {
            (Path::new(url).extension().unwrap().to_string_lossy().into_owned(), false)
        } else {
            ([size_str.as_ref(), ".png"].concat(), true)
        };
        let cached_name = Path::new(Path::new(url).file_name().unwrap())
                .with_extension(&extension);

        let cached_path = self.resize_cache.join(&cached_name);

        if let Err(_) = fs::metadata(&cached_path) {
            if let Ok(ref mut fout) = File::create(&cached_path) {
                if needs_resize {
                    if let None = image {
                        image = match image::open(path) {
                            Ok(image) => Some(image),
                            Err(e) => {
                                println!("error opening image {:?}: {:?}", path, e);
                                return None;
                            },
                        };
                    }
                    let new_image = image.unwrap()
                        .resize_exact(required_size.w, required_size.h, image::FilterType::Lanczos3);

                    match new_image.save(fout, image::PNG) {
                        Err(e) => {
                            println!("error saving resized image {:?}, {:?}: {:?}", path, cached_path, e);
                            return None;
                        },
                        _ => (),
                    }
                } else {
                    let _ = fs::copy(path, &cached_path);
                }
            }
        }

        Some(ResizeResult {
            path: cached_path.to_path_buf(),
            relative_url: cached_name.to_string_lossy().into_owned(),
            size: required_size,
        })
    }
}
