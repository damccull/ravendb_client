# You need to have a file named `license.env` with your license in a variable named `RAVEN_License`
# It should all be on one line looking similar to:
# RAVEN_License={"Id": "LICENSEID", "Name": "Testing", "Keys": [ ... ]}

param([switch]$DontSetupCluster)

$ErrorActionPreference = 'Stop'

$composeCommand = "docker-compose"
$composeArgs = @()
$composeArgs += "up";
$composeArgs += "--force-recreate"
$composeArgs += "-d"

Invoke-Expression -Command "$composeCommand $composeArgs"

if ($DontSetupCluster) {
    exit 0
}

$nodes = @(
    "http://raven1:8080",
    "http://raven2:8080",
    "http://raven3:8080"
);

function AddNodeToCluster() {
    param($FirstNodeUrl, $OtherNodeUrl, $AssignedCores = 1)

    $otherNodeUrlEncoded = $OtherNodeUrl
    $uri = "$($FirstNodeUrl)/admin/cluster/node?url=$($otherNodeUrlEncoded)&assignedCores=$AssignedCores"
    $curlCmd = "curl -L -X PUT '$uri' -d ''"
    docker exec -it raven1 bash -c "$curlCmd"
    Write-Host
    Start-Sleep -Seconds 10
}


Start-Sleep -Seconds 10 

$firstNodeIp = $nodes[0]
$nodeAcoresReassigned = $false
foreach ($node in $nodes | Select-Object -Skip 1) {
    write-Host "Add node $node to cluster";
    AddNodeToCluster -FirstNodeUrl $firstNodeIp -OtherNodeUrl $node

    if ($nodeAcoresReassigned -eq $false) {
        write-host "Reassign cores on A to 1"
        $uri = "$($firstNodeIp)/admin/license/set-limit?nodeTag=A&newAssignedCores=1"
        $curlCmd = "curl -L -X POST '$uri' -d ''"
        docker exec -it raven1 bash -c "$curlCmd"
    }

}

write-host "These run on DockerDesktop, so they are available under one IP - usually 127.0.0.1, so:"
write-host "raven1 localhost:8081"
write-host "raven2 localhost:8082"
write-host "raven3 localhost:8083"
