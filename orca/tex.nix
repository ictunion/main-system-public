{ texlive }:
texlive.combine {
  inherit (texlive)
    scheme-small
    datetime
    fmtcount;
}
