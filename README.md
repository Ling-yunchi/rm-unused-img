# rm-unused-img

this is a tool to remove unused images from your markdown image directory.

it is useful when you use Editor like Typora and set auto copy image to a directory.

## Usage

1. select the markdown file you want to check.
   - support md style image: `![image](./image.png)` and html style image: `<img src="./image.png" />`
   - only support **relative path** image
2. it will auto check asset directory of the markdown file. if markdown file is `./${filename}.md`, it will check
   - `./${filename}`
   - `./${filename}.assets`
3. if auto check failed, or you want to check other directory, you can input the directory manually.
4. then the images in the directory will be listed. green is used, red is unused.
5. click remove button to remove unused images.
6. click refresh button to refresh the list.
7. click rename button to rename the image.
   - rename the images with 1.xxx, 2.xxx, 3.xxx, ...

## Preview

![preview](./preview.png)
