# tinify

> 🚧 Attention! This is a work in progress. The API is not stable and may change at any time.

The goal is to provide better compression than [tinypng](https://tinypng.com/).

## Usage
GET /ping - check if the server is running

POST /compress - compress an image (multipart/form-data)
- image: the image to compress
- quality: the quality of the compressed image (0-100) (default: 70)

## Supported formats
- png
- jpg