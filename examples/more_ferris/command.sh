#!/bin/env bash
set -e

# you need ffmpeg and imagemagick (for convert and mogrify)


# get animated ferris from: https://mir-s3-cdn-cf.behance.net/project_modules/disp/fe36cc42774743.57ee5f329fae6.gif
# or here (permalink): https://www.behance.net/gallery/42774743/Rustacean/modules/260962025
# and name it more_ferris.gif
echo ">>> get animated ferris"
wget -O more_ferris.gif https://mir-s3-cdn-cf.behance.net/project_modules/disp/fe36cc42774743.57ee5f329fae6.gif

echo ">>> deconstructing GIF"
convert -coalesce more_ferris.gif more_ferris.png

# image scale is 540x540, cropping the vertical middle part to a 2:1 image
echo ">>> cropping"
mogrify -crop 540x270+0+135 *.png

# longan nano display has 160x80 pixel (2:1 ratio)
echo ">>> resizing"
mogrify -resize 160x80 *.png

rm -f more_ferris.raw tmp.raw
touch more_ferris.raw

SKIP=n
while IFS= read -r -d $'\0' FILE; do
	# skip evey second frame to speed up the animation
	if [[ $SKIP == y ]]; then
		echo ">>> skipping $FILE"
		rm "./$FILE"
		SKIP=n
		continue
	else
		SKIP=y
	fi

	# convert to RGB565 format
	echo ">>> converting $FILE"
	ffmpeg -v quiet -vcodec png -i "./$FILE" -vcodec rawvideo -f rawvideo -pix_fmt rgb565 tmp.raw
	rm "./$FILE"

	# concat the raw data every frame will have 160x80x2 byte (rgb565 uses 5+6+5 bit = 2 byte total for RGB)
	cat tmp.raw >> more_ferris.raw
	rm tmp.raw
	echo -n ">>> size now: "
	du -b more_ferris.raw
done < <(find . -name "more_ferris-*.png" -print0 | sort -z -V)

echo "Done: ./more_ferris.raw"
echo "Put it on a micro sdcard (root folder) and insert in your Longan Nano."
