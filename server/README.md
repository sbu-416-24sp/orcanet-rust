# API

Send requests to http://localhost:3000

## History
### GET /get-history
- Params: none
- Returns the history of completed jobs.

### PUT /remove-from-history
- Body: { jobID: String }
- Removes the job with jobID from the history.
- Returns with status 404 if the job is not found, or status 200 if successful.

### PUT /clear-history
- Body: none
- Clears the history of completed jobs.
- Returns with status 200 if successful.


## Jobs
### PUT /add-job
- Body: { fileHash: String, peerId: String }
- Adds and starts a job.
- Returns with status 503 if a problem occurs, or status 200 with the newly created jobID if successful.

### GET /find-peer/:fileHash
- Params: fileHash
- Finds peers who have the file with fileHash.
- Returns status 503 if a problem occurs, or status 200 with a list of peers.

### GET /job-list
- Params: none
- Returns the list of jobs

### GET /job-info/:jobID
- Params: jobID
- Finds info about the job with jobID
- Returns status 404 if no job is found, or status 200 with information about the job.

### GET /job-peer/:jobId
- Params: jobID
- Finds info about the peer of the job with jobID
- Returns status 404 if no job is found, or status 200 with information about the peer of the job.

### PUT /start-jobs
- Body: { jobIDs: String[] }
- Starts the jobs provided
- Returns with status 404 if any of the jobs are not found, or status 200 if successful.
- Warning: If one of the jobIDs is not found, the request will prematurely return and remaining jobs, even if valid, will not be started. The return message will be `Job {job_id} not found` if you want to make another request with jobs after this.

### PUT /pause-jobs
- Body: { jobIDs: String[] }
- Pauses the jobs provided
- Returns with status 404 if any of the jobs are not found, or status 200 if successful.

### PUT /terminate-jobs
- Body: { jobIDs: String[] }
- Terminates the jobs provided
- Returns with status 404 if any of the jobs are not found, or status 200 if successful.


## Files
### GET /file/:hash/info
- Params: hash
- Gets info about the file with hash.
- Returns status 503 if a problem occurs, or status 200 with information about the file.

### POST /upload
- Body: { filePath: String, price: i64 }
- Uploads a file to the server
- Returns status 500 if a problem occurs, or status 200 with the file's hash if successful.


### DELETE /file/:hash
- Params: hash
- Deletes the file with hash
- Returns status 500 if a problem occurs, or status 200 if successful.


## Peers
### GET /get-peer/:peer_id
- Params: peer_id
- Gets info about a peer
- Returns status 404 if the peer is not found, or status 200 with the peer's info if successful.

### GET /get-peers
- Params: none
- Returns a list of all peers

### POST /remove-peer
- Body: { peer_id: String }
- Removes a peer
- Returns with status 500 if a problem occurs, or status 200 if successful
