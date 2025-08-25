# Create OpenTelemetry Overview Image

## Setup

Install LaTeX and ImageMagick.
```shell
apt-get install -y texlive-latex-base imagemagick
apt-get install -y texlive-full  # For full LaTeX support, optional
```

## Generate Image

First, compile the LaTeX file to PDF, then convert the PDF to a PNG image.
```shell
pdflatex opentelemetry-overview.tex
convert -density 300 opentelemetry-overview.pdf -quality 90 opentelemetry-overview.png
```

## Logos

Note: The use of third-party logos in architecture diagrams is solely for informational and attribution purposes. Their inclusion does not imply endorsement or partnership with any of the respective organizations. All trademarks and logos remain the property of their respective owners.
