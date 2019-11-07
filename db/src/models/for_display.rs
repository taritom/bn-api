use utils::errors::*;

pub trait ForDisplay<U> {
    fn for_display(self) -> Result<U, DatabaseError>;
}

impl<U, T> ForDisplay<U> for Result<T, DatabaseError>
where
    T: ForDisplay<U>,
{
    fn for_display(self) -> Result<U, DatabaseError> {
        match self {
            Ok(r) => r.for_display(),
            Err(e) => Err(e),
        }
    }
}

impl<U, T> ForDisplay<Vec<U>> for Vec<T>
where
    T: ForDisplay<U>,
{
    fn for_display(self) -> Result<Vec<U>, DatabaseError> {
        let mut res = Vec::<U>::new();
        for i in self {
            res.push(i.for_display()?);
        }
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(PartialEq, Debug)]
    struct R {
        id: String,
        x: String,
    }

    impl Clone for R {
        fn clone(&self) -> Self {
            R {
                id: self.id.clone(),
                x: self.x.clone(),
            }
        }
    }

    #[derive(PartialEq, Debug)]
    struct DisplayR {
        x: String,
    }

    impl ForDisplay<DisplayR> for R {
        fn for_display(self) -> Result<DisplayR, DatabaseError> {
            Ok(DisplayR { x: self.x })
        }
    }

    #[test]
    fn for_display() {
        let e = R {
            id: "test".to_string(),
            x: "test".to_string(),
        };
        assert_eq!(Ok(e.clone()).for_display(), Ok(DisplayR { x: "test".to_string() }));

        assert_eq!(e.for_display(), Ok(DisplayR { x: "test".to_string() }));
    }

    #[test]
    fn for_display_vec() {
        let e = vec![
            R {
                id: "test".to_string(),
                x: "test".to_string(),
            },
            R {
                id: "test2".to_string(),
                x: "test2".to_string(),
            },
        ];

        assert_eq!(
            e.clone().for_display(),
            Ok(vec![
                DisplayR { x: "test".to_string() },
                DisplayR { x: "test2".to_string() }
            ])
        );
        assert_eq!(
            Ok(e.clone()).for_display(),
            Ok(vec![
                DisplayR { x: "test".to_string() },
                DisplayR { x: "test2".to_string() }
            ])
        )
    }
}
