#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct SourcePos {
    pub line: usize,
    pub col: usize,
}

impl SourcePos {
    fn start() -> Self {
        Self { line: 1, col: 1 }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct Span {
    pub start: SourcePos,
    pub end: SourcePos,
}

/*
-------------
--- Spans ---
-------------

data Span = Span {start :: AlexSourcePos, end :: AlexSourcePos}
  deriving (Show, Read, Eq, Ord, Generic)

emptySpan :: Span
emptySpan = Span (AlexSourcePos 0 0) (AlexSourcePos 0 0)

instance Semigroup Span where
  (Span s1 e1) <> (Span s2 e2) = Span (min s1 s2) (max e1 e2)

overStart :: (AlexSourcePos -> AlexSourcePos) -> Span -> Span
overStart f (Span start end) = Span (f start) end

overEnd :: (AlexSourcePos -> AlexSourcePos) -> Span -> Span
overEnd f (Span start end) = Span start (f end)

setStart :: AlexSourcePos -> Span -> Span
setStart sp (Span _ end) = Span sp end

setEnd :: AlexSourcePos -> Span -> Span
setEnd sp (Span start _) = Span start sp

-----------------
--- Locations ---
-----------------

-- | The product of `a` and a `Span` representing `a`'s source position.
data Loc a = Loc {getSpan :: Span, unLoc :: a}
  deriving (Show, Eq, Ord, Functor, Generic)

instance Semigroup a => Semigroup (Loc a) where
  Loc s1 a1 <> Loc s2 a2 = Loc (s1 <> s2) (a1 <> a2)

overSpan :: (Span -> Span) -> Loc a -> Loc a
overSpan f (Loc s a) = Loc (f s) a

setSpan :: Span -> Loc a -> Loc a
setSpan s loc = overSpan (const s) loc

-- | The class of types from which we can extract a `Span`
class Located a where
  locate :: a -> Span

instance Located Span where
  {-# INLINE locate #-}
  locate x = x

instance Located (Loc a) where
  {-# INLINE locate #-}
  locate = getSpan

instance (Located a, Located b) => Located (Either a b) where
  {-# INLINE locate #-}
  locate = \case
    Left a -> locate a
    Right b -> locate b
*/
