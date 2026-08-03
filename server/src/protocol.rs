use std::io::{self, Read, Write};

pub const MAX_LENGTH: u32 = 10 * 1024 * 1024;
pub const MAX_PARAM_LEN: u32 = 1024;

pub struct Request {
    pub algorithm_id: u8,
    pub coordinate: u64,
    pub length: u32,
    pub params: Vec<u8>,
}

impl Request {
    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        w.write_all(&[self.algorithm_id])?;
        w.write_all(&self.coordinate.to_be_bytes())?;
        w.write_all(&self.length.to_be_bytes())?;
        w.write_all(&(self.params.len() as u32).to_be_bytes())?;
        w.write_all(&self.params)?;
        Ok(())
    }
}

pub fn read_request<R: Read>(r: &mut R) -> io::Result<Request> {
    let mut algo_buf = [0u8; 1];
    r.read_exact(&mut algo_buf)?;

    let mut coord_buf = [0u8; 8];
    r.read_exact(&mut coord_buf)?;
    let coordinate = u64::from_be_bytes(coord_buf);

    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let length = u32::from_be_bytes(len_buf);
    if length > MAX_LENGTH {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "requested length exceeds maximum",
        ));
    }

    let mut param_len_buf = [0u8; 4];
    r.read_exact(&mut param_len_buf)?;
    let param_len = u32::from_be_bytes(param_len_buf);
    if param_len > MAX_PARAM_LEN {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "params too large"));
    }

    let mut params = vec![0u8; param_len as usize];
    r.read_exact(&mut params)?;

    Ok(Request {
        algorithm_id: algo_buf[0],
        coordinate,
        length,
        params,
    })
}

#[derive(Debug)]
pub enum Response {
    Ok(Vec<u8>),
    Err(String),
}

impl Response {
    pub fn write_to<W: Write>(&self, w: &mut W) -> io::Result<()> {
        match self {
            Response::Ok(data) => {
                w.write_all(&[0u8])?;
                w.write_all(&(data.len() as u32).to_be_bytes())?;
                w.write_all(data)?;
            }
            Response::Err(msg) => {
                w.write_all(&[1u8])?;
                let bytes = msg.as_bytes();
                w.write_all(&(bytes.len() as u32).to_be_bytes())?;
                w.write_all(bytes)?;
            }
        }
        Ok(())
    }
}

pub fn read_response<R: Read>(r: &mut R) -> io::Result<Response> {
    let mut status = [0u8; 1];
    r.read_exact(&mut status)?;

    let mut len_buf = [0u8; 4];
    r.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut data = vec![0u8; len];
    r.read_exact(&mut data)?;

    match status[0] {
        0 => Ok(Response::Ok(data)),
        _ => Ok(Response::Err(String::from_utf8_lossy(&data).into_owned())),
    }
}
