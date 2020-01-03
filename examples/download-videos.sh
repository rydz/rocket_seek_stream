# Get some videos for the example
# Depends on youtube-dl for retrieving the videos.

dl() {
    youtube-dl --flat-playlist $1 -o $2
}

dl "https://youtu.be/VQ4GXz-85Hw" "kosmodrom.webm"
dl "https://youtu.be/A4OL4s3TWj4" "fly_me_to_the_moon.webm"
dl "https://youtu.be/mjoAx9djpNQ" "cruel_angels_thesis.webm"
dl "https://youtu.be/HaMP1tWFo9Y" "ison.webm"