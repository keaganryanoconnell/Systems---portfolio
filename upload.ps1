$ftpHost = "145.223.89.17"
$username = "u671349570"
$password = "Waterpolo123!321"
$localDir = "C:\Users\keaga\OneDrive\Documents\Main Project App\ui-control-center\out"
$remoteBase = "public_html"

function Upload-FtpFile($localPath, $remotePath) {
    $uri = "ftp://$ftpHost/$remotePath"
    $request = [System.Net.FtpWebRequest]::Create($uri)
    $request.Method = [System.Net.WebRequestMethods+Ftp]::UploadFile
    $request.Credentials = New-Object System.Net.NetworkCredential($username, $password)
    $request.UsePassive = $true
    $request.UseBinary = $true
    $request.KeepAlive = $false

    $content = [System.IO.File]::ReadAllBytes($localPath)
    $request.ContentLength = $content.Length

    try {
        $stream = $request.GetRequestStream()
        $stream.Write($content, 0, $content.Length)
        $stream.Close()
        $stream.Dispose()

        $response = $request.GetResponse()
        Write-Output "  OK: $remotePath"
        $response.Close()
    } catch {
        Write-Output "  FAIL: $remotePath - $_"
    }
}

function Create-FtpDirectory($remoteDir) {
    try {
        $uri = "ftp://$ftpHost/$remoteDir"
        $request = [System.Net.FtpWebRequest]::Create($uri)
        $request.Method = [System.Net.WebRequestMethods+Ftp]::MakeDirectory
        $request.Credentials = New-Object System.Net.NetworkCredential($username, $password)
        $request.UsePassive = $true
        $request.KeepAlive = $false
        $response = $request.GetResponse()
        $response.Close()
        Write-Output "  DIR: $remoteDir"
    } catch {
    }
}

function Upload-Directory($localPath, $remotePath) {
    Create-FtpDirectory $remotePath

    Get-ChildItem $localPath | ForEach-Object {
        if ($_.PSIsContainer) {
            $subRemote = "$remotePath/$($_.Name)" -replace '\\', '/'
            Upload-Directory $_.FullName $subRemote
        } else {
            $path = "$remotePath/$($_.Name)" -replace '\\', '/'
            Upload-FtpFile $_.FullName $path
        }
    }
}

Write-Output "Uploading contents of $localDir to $remoteBase on $ftpHost ..."
Upload-Directory $localDir $remoteBase
Write-Output "Done."
