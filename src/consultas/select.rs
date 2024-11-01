use crate::{condiciones, errors, manejo_de_csv, manejo_de_string};
use errors::SqlError;
///Funcion para ordenar las lineas cuando se hace un SELECT
/// Recibe la matriz, el header y un vector ordenamiento, con la condicion y si es ASC o DESC
fn ordenar_matriz(
    matriz: Vec<Vec<String>>,
    ordenamiento: Vec<String>,
    header: &[String],
) -> Result<Vec<Vec<String>>, SqlError> {
    let mut matriz = matriz;
    let fila_1 = matriz.remove(0);

   
    let pos = match manejo_de_csv::obtener_posicion_header(&ordenamiento[0].to_lowercase(), header)
    {
        Ok(pos) => pos,

        Err(e) => {
            return Err(e);
        }
    };


    if ordenamiento[1] == "ASC" {
        matriz.sort_by(|a, b| a[pos].cmp(&b[pos]));
    } else if ordenamiento[1] == "DESC" {
        matriz.sort_by(|a, b| b[pos].cmp(&a[pos]));
    }

    else {
        return Err(errors::SqlError::InvalidSyntax);
    }

    matriz.insert(0, fila_1);

    Ok(matriz)
}
///Funcion para mostrar las columnas que se piden durante el SELECT
///Segun las columnas seleccionadas en el SELECT recibe la matriz previamente armada y muestra en orden dichas columnas con sus datos.
fn mostrar_select(
    matriz: Vec<Vec<String>>,
    columnas_selec: String,
    header: &[String],
    ordenamiento: Vec<String>,
) {
    let columnas_selec: Vec<String> = columnas_selec
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let mut posiciones: Vec<usize> = Vec::new();

    for valor in &columnas_selec {
        
        match manejo_de_csv::obtener_posicion_header(valor, header) {
            Ok(pos) => posiciones.push(pos),

            Err(e) => {
                println!("{}", e);
                return;
            }
        };
    }
   
    let matriz = match ordenar_matriz(matriz, ordenamiento, header) {
        Ok(matriz) => matriz,

        Err(e) => {
            println!("{}", e);
            return;
        }
    };
   
    for fila in &matriz {
        let fila_ord: Vec<String> = posiciones.iter().map(|&i| fila[i].to_string()).collect();

        println!("{}", fila_ord.join(","));
    }
}

///Funcion que se encarga de manejar la consulta "SELECT"
/// Recibe la consulta y la ruta del archivo y llama a las demas funciones para procesarlos y realizar el SELECT
pub fn select(consulta_sql: &str, ruta_del_archivo: &str) -> Result<(), SqlError> {
    let (nombre_csv, mut columnas, condiciones) =
        match manejo_de_string::separar_datos_select(&consulta_sql) {
            Ok((nombre_csv, columnas, condiciones)) => (nombre_csv, columnas, condiciones),

            Err(e) => {
                return Err(e);
            }
        };

    let (condiciones, ordenamiento) = match  manejo_de_string::separar_order(condiciones){

        Ok((condiciones, ordenamiento)) => (condiciones, ordenamiento),
        Err(e) => {
            return Err(e);
        }
    };

    let condiciones_parseadas = match condiciones::procesar_condiciones(condiciones) {
        Ok(condiciones) => condiciones,
        Err(e) => {
            return Err(e);
        }
    };
    let ruta_csv = manejo_de_csv::obtener_ruta_del_csv(ruta_del_archivo, &nombre_csv);

    let header = match manejo_de_csv::leer_header(&ruta_csv, 0) {
        Ok(header) => header,

        Err(e) => {
            return Err(e);
        }
    };

   
    let matriz = match condiciones::comparar_con_csv(condiciones_parseadas, ruta_csv, &header) {
        Ok(matriz) => matriz,

        Err(e) => {
            return Err(e);
        }
    };
    
    if columnas == "*" {
        columnas = header.join(",");
    }
    
    mostrar_select(matriz, columnas, &header, ordenamiento);
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{remove_file, File},
        io::{BufRead, BufReader, BufWriter, Write},
        process::{Command, Stdio},
    };

    use crate::consultas::lock_test::{_acquire_lock,_release_lock};
    #[test]
    fn realizo_un_select_con_varias_condiciones() {
        _acquire_lock();
        //Para los test del select uso el archivo ordenes.csv
        let output = Command::new("cargo")
            .arg("run")
            .arg("--")
            .arg("Archivos_Csv")
            .arg("SELECT * FROM ordenes WHERE producto = Monitor ORDER BY CANTIDAD ASC;")
            .stdout(Stdio::piped()) // Redirigir stdout
            .output()
            .unwrap();

        let out = "output.csv";
        let archivo = File::create(out).unwrap();
        let mut writer = BufWriter::new(archivo);

        if output.status.success() {
            // Convertir la salida en un String
            let stdout = String::from_utf8_lossy(&output.stdout);
            //Una vez ue tengo el stdout le quito la parte de "Archivos_Csv" y la sel select y luego lo inserto en el archivo
            let stdout = stdout.replace("Archivos_Csv", "");
            let stdout = stdout.replace(
                "SELECT * FROM ordenes WHERE producto = Monitor ORDER BY CANTIDAD ASC;",
                "",
            );

            writeln!(writer, "{}", stdout.trim()).unwrap();
            writer.flush().unwrap();
        }

        //leo linea a linea el archivo output.csv con un vector y lo meto en un vector para luego hacer un assert
        let archivo = File::open(out).expect("No se pudo abrir el archivo");
        let lector = BufReader::new(archivo);
        let mut lineas = lector.lines();
        let mut vec: Vec<String> = Vec::new();

        //salteo la primera linea para no leer el header
        lineas.next();

        for linea in lineas {
            vec.push(linea.expect("No se pudo leer la linea"));
        }
        assert_eq!(
            vec,
            vec!["2222,2,Monitor,1", "1,2,Monitor,22", "3333,3,Monitor,22"]
        );

        remove_file(out).expect("No se pudo eliminar el archivo");

        _release_lock();
    }

    #[test]
    fn realizo_un_insert_con_varias_condiciones() {
        _acquire_lock();
        //Para los test del select uso el archivo ordenes.csv
        let output = Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("Archivos_Csv")
        .arg("SELECT * FROM ordenes WHERE producto = Monitor AND id > 222 NOT id_cliente = 2 ORDER BY CANTIDAD ASC;")
        .stdout(Stdio::piped())  // Redirigir stdout
        .output().unwrap();

        let out = "output.csv";
        let archivo = File::create(out).unwrap();
        let mut writer = BufWriter::new(archivo);

        if output.status.success() {
            // Convertir la salida en un String
            let stdout = String::from_utf8_lossy(&output.stdout);
            //Una vez ue tengo el stdout le quito la parte de "Archivos_Csv" y la sel select y luego lo inserto en el archivo
            let stdout = stdout.replace("Archivos_Csv", "");
            let stdout = stdout.replace("SELECT * FROM ordenes WHERE producto = Monitor AND id > 222 NOT id_cliente = 2 ORDER BY CANTIDAD ASC;", "");

            writeln!(writer, "{}", stdout.trim()).unwrap();
            writer.flush().unwrap();
        }

        //leo linea a linea el archivo output.csv con un vector y lo meto en un vector para luego hacer un assert
        let archivo = File::open(out).expect("No se pudo abrir el archivo");
        let lector = BufReader::new(archivo);
        let mut lineas = lector.lines();
        let mut vec: Vec<String> = Vec::new();

        //salteo la primera linea para no leer el header
        lineas.next();

        for linea in lineas {
            vec.push(linea.expect("No se pudo leer la linea"));
        }
        assert_eq!(vec, vec!["3333,3,Monitor,22"]);

        remove_file(out).expect("No se pudo eliminar el archivo");
        _release_lock();
    }
}