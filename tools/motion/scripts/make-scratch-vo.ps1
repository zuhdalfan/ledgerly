# Generates a scratch voiceover with the Windows SAPI voice, so the edit can be
# timed before a human read exists.
#
#   npm run vo:scratch
#
# This is a placeholder and it sounds like one. Ship a real read: drop the wavs
# into public/audio/vo/ with the same names and they win over these files.
# Windows-only; the render itself needs no speech engine.
#
# Keep this file ASCII: Windows PowerShell 5.1 reads a BOM-less script as ANSI,
# and a stray em dash becomes a quote that breaks the parse.

param([string]$Voice = "Microsoft Hazel Desktop")

Add-Type -AssemblyName System.Speech

$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$source = Get-Content -Raw (Join-Path $here "..\src\vo.ts")
$outDir = Join-Path $here "..\public\audio\vo-scratch"
New-Item -ItemType Directory -Force -Path $outDir | Out-Null

$pattern = 'id:\s*"([^"]+)",\s*beat:\s*"[^"]+",\s*local:\s*\d+,\s*duration:\s*\d+,\s*text:\s*"([^"]+)"'
$found = [regex]::Matches($source, $pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)

if ($found.Count -eq 0) {
  Write-Error "No VO lines found in src/vo.ts - did the shape change?"
  exit 1
}

$synth = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {
  $synth.SelectVoice($Voice)
} catch {
  Write-Warning "Voice '$Voice' not available; using the system default."
}
$synth.Rate = 4
$synth.Volume = 100

foreach ($item in $found) {
  $id = $item.Groups[1].Value
  $text = $item.Groups[2].Value
  $path = Join-Path $outDir ($id + ".wav")
  $synth.SetOutputToWaveFile($path)
  $synth.Speak($text)
  $synth.SetOutputToNull()
  Write-Output ("vo: " + $id + " -> " + (Split-Path -Leaf $path))
}

$synth.Dispose()
Write-Output "Scratch voiceover written. Replace with a real read before publishing."
