use ::webtoon::platform::webtoons::Type;

pub mod creator;
pub mod episodes;
pub mod webtoon;

pub trait FromWtType<T> {
    fn from_wt_type(value: T) -> Self;
    fn to_local_type(&self) -> T;
}

impl FromWtType<Type> for webtoon_sdk::WtType {
    fn from_wt_type(value: Type) -> Self {
        match value {
            Type::Original => Self::Original,
            Type::Canvas => Self::Canvas,
        }
    }

    fn to_local_type(&self) -> Type {
        match self {
            webtoon_sdk::WtType::Canvas => Type::Canvas,
            webtoon_sdk::WtType::Original => Type::Original,
        }
    }
}
