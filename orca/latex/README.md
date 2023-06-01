We use xelatex to print PDFs. You can even print pdfs directly from this directory but first you need to configure local:

```
ln -s lang.end.tex lang.tex
```

then you should be able to run:

```
xelatex registration.tex
```

You also need to have opentype IBM/Plex fonts in your system. You can download them on [Github](https://github.com/IBM/plex/releases/latest)
