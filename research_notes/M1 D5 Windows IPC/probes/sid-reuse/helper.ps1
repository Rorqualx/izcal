# helper.ps1: the PasswordVault (Credential Locker) channel for the izq
# probe. Runs inside whatever container spawns it (or as the plain user).
#   helper.ps1 plant-vault <resfile> <resource>
#   helper.ps1 check-vault <resfile> <resource>
#   helper.ps1 remove-vault <resfile> <resource>
# Writes exactly one key=value line: vault_new= / vault= / vault_remove=.
param([string]$Mode, [string]$ResFile, [string]$Resource)
$ErrorActionPreference = 'Stop'
function Emit($s) { Set-Content -Path $ResFile -Value $s -Encoding ASCII }
try {
  $null = [Windows.Security.Credentials.PasswordVault,Windows.Security.Credentials,ContentType=WindowsRuntime]
  $v = New-Object Windows.Security.Credentials.PasswordVault
  if ($Mode -eq 'plant-vault') {
    $c = New-Object Windows.Security.Credentials.PasswordCredential
    $c.Resource = $Resource; $c.UserName = 'izq-user'; $c.Password = 'izq-pass'
    $v.Add($c)
    Emit 'vault_new=ok'
  } elseif ($Mode -eq 'check-vault') {
    $hit = $false
    foreach ($c in $v.RetrieveAll()) { if ($c.Resource -eq $Resource) { $hit = $true } }
    if ($hit) { Emit 'vault=found' } else { Emit 'vault=gone' }
  } else {
    $hit = $null
    foreach ($c in $v.RetrieveAll()) { if ($c.Resource -eq $Resource) { $hit = $c } }
    if ($hit) { $v.Remove($hit); Emit 'vault_remove=ok' } else { Emit 'vault_remove=absent' }
  }
} catch {
  $hr = 0; if ($_.Exception.InnerException) { $hr = $_.Exception.InnerException.HResult }
  if ($hr -eq 0) { $hr = $_.Exception.HResult }
  $m = $_.Exception.Message; if ($m.Length -gt 120) { $m = $m.Substring(0, 120) }
  $k = 'vault'; if ($Mode -eq 'plant-vault') { $k = 'vault_new' }; if ($Mode -eq 'remove-vault') { $k = 'vault_remove' }
  Emit ("{0}=DENIED(0x{1:x}) {2}" -f $k, $hr, $m)
}
