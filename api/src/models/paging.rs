#[derive(Serialize, Deserialize, Clone, Copy)]
///enum used to indicate if query data must be sorted in ascending or descending order
pub enum SortingDir {
    Asc,
    Desc,
    None,
}

#[derive(Serialize, Deserialize, Clone)]
///struct used to indicate paging information and search query information
pub struct Paging {
    pub page: u64,
    pub limit: u64,
    pub sort: String,
    pub dir: SortingDir,
    pub total: u64,
    pub tags: Vec<SearchParams>,
}

#[derive(Serialize, Deserialize, Clone)]
///Struct used to store search object names and values
pub struct SearchParams {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Serialize)]
///return wrapper struct for returning large lists
pub struct Payload<T> {
    pub data: Vec<T>,
    pub paging: Paging,
}

#[derive(Serialize, Deserialize, Clone)]
///struct used to indicate paging information and search query information
pub struct PagingParameters {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub sort: Option<String>,
    pub dir: Option<SortingDir>,
    pub tags: Option<Vec<SearchParams>>,
}

impl Paging {
    pub fn new(received: &PagingParameters) -> Self {
        let default_page = if let Some(i) = received.page { i } else { 0 };
        let default_limit = if let Some(i) = received.limit { i } else { 100 };
        let default_sort = if let Some(ref i) = received.sort {
            i.clone()
        } else {
            ("").to_owned()
        };
        let default_dir = if let Some(i) = received.dir {
            i
        } else {
            SortingDir::None
        };
        let default_tags = if let Some(ref i) = received.tags {
            i.clone()
        } else {
            Vec::new()
        };
        Paging {
            page: default_page,
            limit: default_limit,
            sort: default_sort,
            dir: default_dir,
            total: 0,
            tags: default_tags,
        }
    }

    pub fn clone_with_new_total(received: &Paging, new_total: u64) -> Self {
        Paging {
            page: received.page.clone(),
            limit: received.limit.clone(),
            sort: received.sort.clone(),
            dir: received.dir.clone(),
            total: new_total,
            tags: received.tags.clone(),
        }
    }
}
