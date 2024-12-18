use crate::{
    errors::{self, SqlError},
    manejo_de_csv::{self},
    tipo_de_datos,
};
/// Retorno true si el parentesis derecho
pub fn parentesis_izquierdo(token: &str) -> bool {
    token == "("
}

/// Retorna false si es el parentesis derecho
pub fn partentesis_derecho(token: &str) -> bool {
    token == ")"
}
///Funcion para separar las condiciones de una consulta
/// #Recibe por parametro un string con las condiciones
/// -Crea un vector de strings vacio
/// -Crea un string vacio para almacenar la palabra actual
/// -Itera sobre las condiciones
/// -Si el caracter es un espacio y la palabra actual no esta vacia la agrega al vector
/// -Si el caracter es un parentesis lo agrega al vector
/// -Si no es un espacio o un parentesis lo agrega a la palabra actual
/// -Si la palabra actual no esta vacia la agrega al vector
/// -Retorna el vector con las condiciones separadas o en otro caso un vector vacio
pub fn separar_condiciones(condiciones: &str) -> Vec<String> {
    let mut condiciones_separadas: Vec<String> = Vec::new();
    let mut palabra_actual = String::new();

    for c in condiciones.chars() {
        if c.is_whitespace() {
            if !palabra_actual.is_empty() {
                condiciones_separadas.push(palabra_actual.clone());
                palabra_actual.clear();
            }
        } else if c == '(' || c == ')' {
            if !palabra_actual.is_empty() {
                condiciones_separadas.push(palabra_actual.clone());
                palabra_actual.clear();
            }

            condiciones_separadas.push(c.to_string());
        } else {
            palabra_actual.push(c);
        }
    }

    if !palabra_actual.is_empty() {
        condiciones_separadas.push(palabra_actual);
    }

    condiciones_separadas
}

///Funcion para obtener la primera palabra de nuestra consulta
///#Recibe una cadena por parametro con la consulta completa
///-Divide la cadena por espacios
///-Devuelve la primera palabra encontrada
///-En otro caso devuelve un string vacio (no hice que devuelva un error por que se maneja en el main)
pub fn obtener_primera_palabra(cadena: &str) -> Result<String, SqlError> {
    let mut iterar_en_cadena = cadena.split_whitespace();

    if let Some(palabra) = iterar_en_cadena.next() {
        Ok(palabra.to_string())
    } else {
        Err(errors::SqlError::InvalidSyntax)
    }
}

///Funcion para separar los datos de la consulta INSERT
///#Recibe por parametro la consulta
///-Define un vector con dos partes, usando VALUES para separar
///-Luego separa esas dos partes y opera para dejar valores y direccione_y_columnas como Strings separados
///-Finalmente retorna los dos Strings
pub fn separar_datos(consulta_sql: &str) -> Result<(String, String, Vec<String>), SqlError> {
    let palabras: Vec<&str> = consulta_sql.split_whitespace().collect();

    if let Some(pos) = palabras.iter().position(|&x| x == "VALUES") {
        if palabras[..pos].join(" ").contains("INTO") {
            let insert = palabras[..pos].join(" ").to_string();
            let valores = palabras[pos + 1..]
                .join(" ")
                .trim_end_matches(';')
                .trim()
                .replace(" ", "")
                .replace("'", "")
                .to_string();

            let mut columnas: Vec<&str> = insert.split_whitespace().collect();
            columnas.drain(0..2); // Quitamos "INSERT" y "INTO"

            let (nombre_csv, columnas) = columnas.split_at(1);
            let nombre_csv = nombre_csv[0].to_string();

            let columnas: Vec<String> = columnas
                .join(",")
                .to_string()
                .replace("(", "")
                .replace(")", "")
                .trim()
                .split(",")
                .map(|s| s.to_string())
                .collect();

            Ok((nombre_csv, valores, columnas))
        } else {
            Err(errors::SqlError::InvalidSyntax)
        }
    } else {
        Err(errors::SqlError::InvalidSyntax)
    }
}

///Funcion para separar los datos de la consulta UPDATE
///#Recibe por parametro la consulta sql
///-Divide la consulta en dos partes una con el nombre del csv y otra con los valores
///-Con la primera cadena que contiene el nombre reemplaza el UPDATE para dejar solo el nombre del csv
///-Con la segunda cadena que contiene los valores itera sobre dicha cadena y separa la clave de los campos a actualizar
///-Finalmente retorn un string con el nombre, un vector con los valores y otro con la clave para actualizar
///-En otro caso devuelve un error
pub fn separar_datos_update(
    consulta_sql: &str,
) -> Result<(String, Vec<String>, Vec<String>), SqlError> {
    let palabras: Vec<&str> = consulta_sql.split_whitespace().collect();

    if palabras.iter().any(|&x| x == "SET") {
        let partes: Vec<&str> = consulta_sql.split("SET").collect();
        let nombre_del_csv = partes[0]
            .trim()
            .replace("UPDATE", "")
            .replace("'", "")
            .replace(" ", "")
            .replace(" ", "")
            .to_string();
        let valores = partes[1].trim().trim_end_matches(';');

        if palabras.iter().any(|&x| x == "WHERE") {
            if let Some((campos_a_actualizar, clave)) = valores.split_once("WHERE") {
                let set_campos: Vec<String> = campos_a_actualizar
                    .split_whitespace()
                    .map(|s| s.to_string().replace("'", ""))
                    .collect();
                //SI encuentro un parente tambien lo separo como si fuera un solo caracter

                let condiciones: Vec<String> = separar_condiciones(clave);
                println!("COndiciones {:?}", condiciones);
                Ok((nombre_del_csv, set_campos, condiciones))
            } else {
                Err(errors::SqlError::InvalidSyntax)
            }
        } else {
            Err(errors::SqlError::InvalidSyntax)
        }
    } else {
        Err(errors::SqlError::InvalidSyntax)
    }
}

///Funcion para separar los datos de la consulta DELETE
///#Recibe por parametro la consulta sql
///-Divide la consulta en dos partes una con el nombre del csv y otra con la clave
///-Con la primera cadena que contiene el nombre reemplaza el DELETE  y el FROM para dejar solo el nombre del csv
///-Con la segunda cadena que contiene los valores itera sobre dicha cadena y deja solamente la calve y el valor a actualizar
///-Finalmente retorn un string con el nombre y un vector clave-valor
///-En otro caso devuelve un error
pub fn separar_datos_delete(consulta_sql: &str) -> Result<(String, Vec<String>), SqlError> {
    let palabras: Vec<&str> = consulta_sql.split_whitespace().collect();

    if palabras.iter().any(|&x| x == "WHERE") && palabras.iter().any(|&x| x == "FROM") {
        
        let partes: Vec<&str> = consulta_sql.split("WHERE").collect();

        let nombre_csv = partes[0]
            .replace("DELETE", "")
            .replace("FROM", "")
            .trim()
            .replace("(", "")
            .replace(")", "")
            .replace(" ", "");
        let condiciones = separar_condiciones(&partes[1].trim_end_matches(';').replace("'", ""));

        Ok((nombre_csv, condiciones))
    } else {
        Err(errors::SqlError::InvalidSyntax)
    }
}

///Funcion para separar los datos de la consulta SELECT
///#Recibe por parametro la consulta sql
///-Divide la consulta en dos partes una con el nombre + columnas y otra con la clave
///-Con la primera cadena que contiene el nombre reemplaza el SELECT  y el FROM para dejar solo el nombre del csv
/// Y en otra variable las columnas del SELECT
///-Con la segunda cadena que contiene las condiciones quita los ; y recoge todo en un vector.
///-Finalmente retorn un string con el nombre otro con las columnas y por ultimo un vector con las condiciones
///-En otro caso devuelve un error
pub fn separar_datos_select(consulta_sql: &str) -> Result<(String, String, Vec<String>), SqlError> {
    let palabras: Vec<&str> = consulta_sql.split_whitespace().collect();
    if palabras.iter().any(|&x| x == "WHERE") {
        if palabras.iter().any(|&x| x == "FROM") {
            let partes: Vec<&str> = consulta_sql.split("WHERE").collect();

            let nombre_csv_y_columnas = partes[0]
                .trim()
                .replace("SELECT", "")
                .replace("'", "")
                .replace("(", "")
                .replace(")", "")
                .replace(" ", "")
                .to_string();

            let nombre_csv_y_columnas: Vec<&str> = nombre_csv_y_columnas.split("FROM").collect();

            let nombre_csv = nombre_csv_y_columnas[1].trim().replace(" ", "").to_string();
            let columnas = nombre_csv_y_columnas[0]
                .trim()
                .replace(" ", "")
                .replace("(", "")
                .replace(")", "")
                .to_string();

            let condiciones = partes[1].trim_end_matches(';').replace("'", "");
            let condiciones = separar_condiciones(&condiciones);

            Ok((nombre_csv, columnas, condiciones))
        } else {
            Err(errors::SqlError::InvalidSyntax)
        }
    } else {
        Err(errors::SqlError::InvalidSyntax)
    }
}

///Funcion para separar el ORDER de las condiciones de un SELECT
///#Recibe por parametro un vector con las condiciones de la consulta
///-Si contiene order itera sobre la cadena y quita el order para almacernar el resto de los strings en un vector
///-Separa el vector en dos strings y luego los mapea en dos vectores que contengan las condiciones y por otro lado el ORDER
///-Finalmente devuelve dos vectores uno con el criterio de ordenamiento y otro con las condiciones
///-En caso de no contener ORDER devuelve un vector con las condiciones y uno de ordenamiento vacio.
pub fn separar_order(condiciones: Vec<String>) -> Result<(Vec<String>, Vec<String>), SqlError> {
    let condiciones = condiciones.join(" ");

    let palabras: Vec<&str> = condiciones.split_whitespace().collect();

    if palabras.iter().any(|&x| x == "ORDER") {
        if palabras.iter().any(|&x| x == "BY") {
            let condiciones = condiciones.split("ORDER BY").collect::<Vec<&str>>();

            let ordenamiento = condiciones[1]
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            let condiciones = condiciones[0];

            let condiciones = separar_condiciones(condiciones);

            Ok((condiciones, ordenamiento))
        } else {
            Err(errors::SqlError::InvalidSyntax)
        }
    } else {
        Err(errors::SqlError::InvalidSyntax)
    }
}

///Funcion para crear una matriz a la hora de utilizar el INSERT con multiples valores
/// #Recibe por parametro los valores, las columnas, el header y la ruta del csv
/// -Reemplaza los parentesis por comas para poder separar los valores
/// -Itera sobre los valores y los separa por comas
/// -Crea un vector con la cantidad de columnas del header y lo llena con strings vacios
/// -Itera sobre los valores y los coloca en la posicion correspondiente
/// -Si no se ingreso una columna se lanza un error
/// -Si el valor ingresado no es correcto se lanza un error
/// -Finalmente devuelve una matriz con los valores ingresados
pub fn crear_matriz(
    valores: String,
    columnas: Vec<String>,
    header: &[String],
    ruta_csv: &String,
) -> Result<Vec<Vec<String>>, SqlError> {
    let valores = valores.replace(")(", "),("); //Por si los valores vienen sin los parentesiss
    let valores: Vec<&str> = valores
        .trim_matches(|c| c == '(' || c == ')')
        .split("),(")
        .collect();

    let mut matriz: Vec<Vec<String>> = Vec::new();
    for valor in valores {
        let vec_sin_ordenar: Vec<String> = valor.split(",").map(|s| s.to_string()).collect();
        let mut vec_ordenado: Vec<String> = Vec::new();
        vec_ordenado.resize(header.len(), "".to_string());

        for (i, elemento) in vec_sin_ordenar.iter().enumerate() {
            //Asumo que si me ingresan una columnas menos, lo añado pero en blanco
            if i < header.len() && i < columnas.len() {
                //Si me escribieron cualquier cosa como columna lanzo un error
                let pos = match manejo_de_csv::obtener_posicion_header(&columnas[i], header) {
                    Ok(pos) => pos,

                    Err(e) => return Err(e),
                };

                let elemento = match tipo_de_datos::comprobar_dato(elemento, ruta_csv, pos) {
                    Ok(elemento) => elemento,

                    Err(e) => return Err(e),
                };

                vec_ordenado[pos] = elemento;
            }
        }

        matriz.push(vec_ordenado);
    }
    Ok(matriz)
}

#[cfg(test)]
mod test {

    use std::vec;

    use super::*;

    #[test]
    fn test05devuelve_la_primera_palabra_de_una_consulta() {
        let consulta = "UPDATE ordenes SET producto = cangrejo WHERE producto = Altavoces ";

        let primera_palabra = match obtener_primera_palabra(&consulta) {
            Ok(palabra) => palabra,
            Err(_) => "".to_string(),
        };

        let palabra_esperada = "UPDATE";
        assert_eq!(primera_palabra, palabra_esperada);
    }
    //El resto de las funciones para separar son muy parecidas
    #[test]
    fn test06separa_los_datos_del_select_y_los_devuelve() {
        let consulta = "SELECT id,producto,cantidad FROM ordenes WHERE producto = Teclado AND cantidad >= 1 ORDER BY CANTIDAD ASC ".to_string();

        let (nombre_csv, columnas, condiciones) = separar_datos_select(&consulta).unwrap();

        let nombre_csv_esperado = "ordenes";
        let columnas_eperadas = "id,producto,cantidad";
        let condiciones_esperadas = vec![
            "producto".to_string(),
            "=".to_string(),
            "Teclado".to_string(),
            "AND".to_string(),
            "cantidad".to_string(),
            ">=".to_string(),
            "1".to_string(),
            "ORDER".to_string(),
            "BY".to_string(),
            "CANTIDAD".to_string(),
            "ASC".to_string(),
        ];

        assert_eq!(nombre_csv, nombre_csv_esperado);
        assert_eq!(columnas, columnas_eperadas);
        assert_eq!(condiciones, condiciones_esperadas);
    }
}
