use parquet::{
    data_type::{ByteArray, ByteArrayType, FloatType, Int32Type, Int64Type},
    file::writer::SerializedRowGroupWriter,
};

/// These methods are bulky, but probably not worth abstracting further.

pub fn write_strs<T: std::io::Write + Send>(
    row_group_writer: &mut SerializedRowGroupWriter<T>,
    values: &[&str],
) -> Result<usize, String> {
    let bytes: Vec<ByteArray> = values.iter().map(|x| (*x).into()).collect();

    let col_writer = row_group_writer.next_column().unwrap();
    if let Some(mut col_writer) = col_writer {
        let result = col_writer
            .typed::<ByteArrayType>()
            .write_batch(&bytes, None, None);
        col_writer.close().unwrap();
        return Ok(result.map_err(|x| x.to_string())?);
    }

    return Err("Invalid column type in parquet schema.".to_string());
}

pub fn write_i32s<T: std::io::Write + Send>(
    row_group_writer: &mut SerializedRowGroupWriter<T>,
    values: &[i32],
) -> Result<usize, String> {
    let col_writer = row_group_writer.next_column().unwrap();
    if let Some(mut col_writer) = col_writer {
        let result = col_writer
            .typed::<Int32Type>()
            .write_batch(values, None, None);
        col_writer.close().unwrap();
        return Ok(result.map_err(|x| x.to_string())?);
    }

    return Err("Invalid column type in parquet schema.".to_string());
}

pub fn write_i64s<T: std::io::Write + Send>(
    row_group_writer: &mut SerializedRowGroupWriter<T>,
    values: &[i64],
) -> Result<usize, String> {
    let col_writer = row_group_writer.next_column().unwrap();
    if let Some(mut col_writer) = col_writer {
        let result = col_writer
            .typed::<Int64Type>()
            .write_batch(values, None, None);
        col_writer.close().unwrap();
        return Ok(result.map_err(|x| x.to_string())?);
    }
    return Err("Invalid column type in parquet schema.".to_string());
}

pub fn write_f32s<T: std::io::Write + Send>(
    row_group_writer: &mut SerializedRowGroupWriter<T>,
    values: &[f32],
) -> Result<usize, String> {
    let col_writer = row_group_writer.next_column().unwrap();
    if let Some(mut col_writer) = col_writer {
        let result = col_writer
            .typed::<FloatType>()
            .write_batch(values, None, None);
        col_writer.close().unwrap();
        return Ok(result.map_err(|x| x.to_string())?);
    }

    return Err("Invalid column type in parquet schema.".to_string());
}
