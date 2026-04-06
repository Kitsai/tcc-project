use crate::problem::ProblemDto;

#[tauri::command]
pub fn create_problem(_name: String, _path: String) -> Result<ProblemDto, String> {
    Ok(ProblemDto {
        path: "Teste".to_string(),
    })
}
