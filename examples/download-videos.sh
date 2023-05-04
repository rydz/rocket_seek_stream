# Get some videos for the example
# Depends on youtube-dl for retrieving the videos.

dl() {
    yt-dlp --flat-playlist $1 -o $2
}

dl "https://youtu.be/iRkTT54L83o" "tari_tari.webm"
dl "https://youtu.be/A4OL4s3TWj4" "fly_me_to_the_moon.webm"
dl "https://youtu.be/k8ozVkIkr-g" "cruel_angels_thesis.webm"
dl "https://youtu.be/HaMP1tWFo9Y" "ison.webm"