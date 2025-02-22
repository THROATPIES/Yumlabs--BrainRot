#!C:/Users/THROATPIES/Documents/Development/ipynb_env_3.9/python.exe
import http.client as httplib
import httplib2
import os
import random
import sys
import time

from googleapiclient.discovery import build
from googleapiclient.errors import HttpError
from googleapiclient.http import MediaFileUpload
from oauth2client.client import flow_from_clientsecrets
from oauth2client.file import Storage
from oauth2client.tools import argparser, run_flow


# Constants
MAX_RETRIES = 10
CLIENT_SECRETS_FILE = "docs\sec.json"
YOUTUBE_UPLOAD_SCOPE = ["https://www.googleapis.com/auth/youtube.upload",
                       "https://www.googleapis.com/auth/youtube", "https://www.googleapis.com/auth/youtube.force-ssl"]
YOUTUBE_API_SERVICE_NAME = "youtube"
YOUTUBE_API_VERSION = "v3"
VALID_PRIVACY_STATUSES = ("public", "private", "unlisted")
DEFAULT_CATEGORY = "22"
DEFAULT_TITLE = "Test Title"
DEFAULT_DESCRIPTION = "Test Description"
DEFAULT_KEYWORDS = ""
PLAYLIST_ID = ""


RETRIABLE_EXCEPTIONS = (
    httplib2.HttpLib2Error,
    IOError,
    httplib.NotConnected,
    httplib.IncompleteRead,
    httplib.ImproperConnectionState,
    httplib.CannotSendRequest,
    httplib.CannotSendHeader,
    httplib.ResponseNotReady,
    httplib.BadStatusLine,
)

RETRIABLE_STATUS_CODES = [500, 502, 503, 504]

MISSING_CLIENT_SECRETS_MESSAGE = """
WARNING: Please configure OAuth 2.0

To make this sample run you will need to populate the client_secrets.json file
found at:

   %s

with information from the API Console
https://console.cloud.google.com/

For more information about the client_secrets.json file format, please visit:
https://developers.google.com/api-client-library/python/guide/aaa_client_secrets
""" % os.path.abspath(os.path.join(os.path.dirname(__file__), CLIENT_SECRETS_FILE))


def get_authenticated_service(args):
    """Get an authenticated YouTube service object."""
    flow = flow_from_clientsecrets(
        CLIENT_SECRETS_FILE, scope=YOUTUBE_UPLOAD_SCOPE, message=MISSING_CLIENT_SECRETS_MESSAGE
    )
    storage = Storage("%s-oauth2.json" % sys.argv[0])
    credentials = storage.get()

    if credentials is None or credentials.invalid:
        credentials = run_flow(flow, storage, args)

    return build(
        YOUTUBE_API_SERVICE_NAME, YOUTUBE_API_VERSION, http=credentials.authorize(httplib2.Http())
    )


def initialize_upload(youtube, options):
    """Initializes and executes the video upload."""
    tags = options.keywords.split(",") if options.keywords else None

    body = dict(
        snippet=dict(
            title=options.title,
            description=options.description,
            tags=tags,
            categoryId=options.category,
        ),
        status=dict(privacyStatus=options.privacyStatus),
    )

    insert_request = youtube.videos().insert(
        part=",".join(body.keys()),
        body=body,
        media_body=MediaFileUpload(options.file, chunksize=-1, resumable=True),
    )

    resumable_upload(insert_request, youtube, options.playlistId)  # Pass the playlist ID


def add_video_to_playlist(youtube, video_id, playlist_id):
    """Adds the uploaded video to the specified playlist."""
    playlist_item_body = {
        "snippet": {
            "playlistId": playlist_id,
            "resourceId": {"kind": "youtube#video", "videoId": video_id},
        }
    }
    try:
        playlist_response = youtube.playlistItems().insert(part="snippet", body=playlist_item_body).execute()
        print(f"Video added to playlist. Playlist item ID: {playlist_response.get('id')}")
    except HttpError as e:
        print(f"An HTTP error {e.resp.status} occurred during playlist addition:\n{e.content}")
        raise  


def resumable_upload(insert_request, youtube, playlist_id):
    """Handles resumable uploads with exponential backoff."""
    response = None
    retry = 0

    while response is None:
        try:
            print("Uploading file...")
            _, response = insert_request.next_chunk()
            if response:
                if "id" in response:
                    video_id = response["id"]
                    print("Video id '%s' was successfully uploaded." % video_id)
                    if playlist_id:  # Only add to playlist if ID is provided
                        add_video_to_playlist(youtube, video_id, playlist_id)
                else:
                    exit("The upload failed with an unexpected response: %s" % response)
        except HttpError as e:
            if e.resp.status in RETRIABLE_STATUS_CODES:
                print(f"A retriable HTTP error {e.resp.status} occurred:\n{e.content}")
            else:
                raise
        except RETRIABLE_EXCEPTIONS as e:
            print(f"A retriable error occurred: {e}")
        
        if response is None:
            retry += 1
            if retry > MAX_RETRIES:
                exit("No longer attempting to retry.")
            sleep_seconds = random.random() * (2**retry)
            print(f"Sleeping {sleep_seconds:.2f} seconds and then retrying...")
            time.sleep(sleep_seconds)


def main():
    """Main function to handle video uploading."""
    print("Attempting upload...")
    argparser.add_argument("--file", required=True, help="Video file to upload")
    argparser.add_argument("--title", help="Video title", default=DEFAULT_TITLE)
    argparser.add_argument("--description", help="Video description", default=DEFAULT_DESCRIPTION)
    argparser.add_argument(
        "--category", default=DEFAULT_CATEGORY, help="Numeric video category."
    )
    argparser.add_argument("--keywords", help="Video keywords, comma separated", default=DEFAULT_KEYWORDS)
    argparser.add_argument(
        "--privacyStatus",
        choices=VALID_PRIVACY_STATUSES,
        default=VALID_PRIVACY_STATUSES[0],
        help="Video privacy status.",
    )
    argparser.add_argument("--playlistId", help="Playlist ID to add the video to")
    args = argparser.parse_args()

    if not os.path.exists(args.file):
        exit("Please specify a valid file using the --file= parameter.")

    youtube = get_authenticated_service(args)
    try:
        initialize_upload(youtube, args)
    except HttpError as e:
        print(f"An HTTP error {e.resp.status} occurred:\n{e.content}")


if __name__ == "__main__":
    httplib2.RETRIES = 1
    main()
