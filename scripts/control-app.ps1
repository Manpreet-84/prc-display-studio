param([switch]$SelfTest,[switch]$Editor)
$ErrorActionPreference='Stop'
Add-Type -AssemblyName PresentationFramework
Add-Type -AssemblyName PresentationCore
Add-Type -AssemblyName WindowsBase
Add-Type -AssemblyName System.Drawing
$script:root=Split-Path $PSScriptRoot -Parent
$script:binary=Join-Path $script:root 'target/release/prc-lcd.exe'
Set-Location $script:root
$env:PRC_THEME_DIR=Join-Path $script:root 'themes'
[xml]$markup=Get-Content (Join-Path $PSScriptRoot 'studio.xaml') -Raw -Encoding UTF8
$reader=New-Object Xml.XmlNodeReader($markup)
$script:window=[Windows.Markup.XamlReader]::Load($reader)
$script:ui=@{}
$ns=New-Object Xml.XmlNamespaceManager($markup.NameTable)
$ns.AddNamespace('x','http://schemas.microsoft.com/winfx/2006/xaml')
foreach($node in $markup.SelectNodes('//*[@x:Name]',$ns)){$name=$node.GetAttribute('Name','http://schemas.microsoft.com/winfx/2006/xaml');$script:ui[$name]=$script:window.FindName($name)}
$script:faces=@();$script:checked=New-Object 'Collections.Generic.HashSet[string]';$script:selectedFace=$null
$script:elements=New-Object Collections.ArrayList;$script:selectedElement=-1;$script:background=$null;$script:sourceImage=$null;$script:dirty=$false;$script:updating=$false
$script:settingsPath=Join-Path $script:root 'app-settings.json'
$script:settings=@{face='custom';rotate=$true;dwell=10;playlist=@()}
if(Test-Path $script:settingsPath){try{$data=Get-Content $script:settingsPath -Raw|ConvertFrom-Json;$script:settings.face=$data.face;$script:settings.rotate=[bool]$data.rotate;$script:settings.dwell=[Math]::Max(2,[Math]::Min(3600,[int]$data.dwell));$script:settings.playlist=@($data.playlist);foreach($id in $data.playlist){$script:checked.Add($id)|Out-Null}}catch{}}
$script:ui.ThemeDwell.Text=[string]$script:settings.dwell;$script:ui.SettingsDwell.Text=[string]$script:settings.dwell
$script:sensors=@(
 @{id='cpu.temp';label='CPU temperature';unit='°C';demo=48;max=100},@{id='cpu.load';label='CPU load';unit='%';demo=22;max=100},@{id='cpu.power';label='CPU power';unit=' W';demo=65;max=250},@{id='cpu.clock';label='CPU clock';unit=' MHz';demo=4800;max=6000},
 @{id='gpu.temp';label='GPU temperature';unit='°C';demo=54;max=100},@{id='gpu.load';label='GPU load';unit='%';demo=78;max=100},@{id='gpu.power';label='GPU power';unit=' W';demo=94;max=450},@{id='gpu.clock';label='GPU clock';unit=' MHz';demo=2400;max=4000},@{id='gpu.fan';label='GPU fan';unit=' RPM';demo=1400;max=5000},
 @{id='ram.load';label='RAM usage';unit='%';demo=47;max=100},@{id='ram.used';label='RAM used';unit=' GB';demo=15;max=64},@{id='vram.load';label='VRAM usage';unit='%';demo=52;max=100},@{id='vram.used';label='VRAM used';unit=' GB';demo=8;max=24},
 @{id='net.down';label='Download';unit=' Mbps';demo=152;max=1000},@{id='net.up';label='Upload';unit=' Mbps';demo=48;max=1000},@{id='cpu.fan';label='CPU fan';unit=' RPM';demo=1200;max=5000},@{id='pump';label='Pump';unit=' RPM';demo=2600;max=5000},
 @{id='disk.temp';label='SSD temperature';unit='°C';demo=38;max=100},@{id='disk.load';label='Disk usage';unit='%';demo=72;max=100},@{id='disk.read';label='Disk read';unit=' MB/s';demo=125;max=1000},@{id='disk.write';label='Disk write';unit=' MB/s';demo=98;max=1000})
foreach($sensor in $script:sensors){$script:ui.SensorSelect.Items.Add([pscustomobject]$sensor)|Out-Null};$script:ui.SensorSelect.SelectedIndex=0
function Notice($text){$script:ui.StatusMessage.Text=$text}
function Safe-Action([scriptblock]$action){try{& $action}catch{Notice ('Could not complete action: '+$_.Exception.Message);Add-Content (Join-Path $script:root 'studio-error.log') $_.Exception.ToString()}}
function Bitmap($path){$b=New-Object Windows.Media.Imaging.BitmapImage;$b.BeginInit();$b.CacheOption=[Windows.Media.Imaging.BitmapCacheOption]::OnLoad;$b.UriSource=New-Object Uri($path);$b.EndInit();$b.Freeze();return $b}
function Persist-Playback { $tmp=$script:settingsPath+'.tmp';[IO.File]::WriteAllText($tmp,($script:settings|ConvertTo-Json -Depth 8));Move-Item -LiteralPath $tmp -Destination $script:settingsPath -Force;Update-Playback }
function Update-Playback {$mode=if($script:settings.rotate){if($script:settings.playlist.Count){"Playlist: $($script:settings.playlist.Count) faces"}else{'All faces rotating'}}else{'Single face'};$script:ui.PlaybackSummary.Text="$mode · $($script:settings.dwell)s per face"}
function Dwell($text){$n=0;if(-not [int]::TryParse($text,[ref]$n) -or $n -lt 2 -or $n -gt 3600){throw 'Enter a whole number from 2 to 3600 seconds.'};return $n}
function Navigate($page){foreach($name in @('Themes','Editor','Settings')){$script:ui[$name+'Page'].Visibility=if($name -eq $page){'Visible'}else{'Collapsed'};$script:ui['Nav'+$name].Background=if($name -eq $page){(New-Object Windows.Media.SolidColorBrush([Windows.Media.ColorConverter]::ConvertFromString('#3DD6F5')))}else{$script:window.FindResource('PanelBrush')};$script:ui['Nav'+$name].Foreground=if($name -eq $page){[Windows.Media.Brushes]::Black}else{[Windows.Media.Brushes]::LightGray}}
 $script:ui.PageTitle.Text=@{Themes='Your display, your way.';Editor='Build a face.';Settings='Make it work for you.'}[$page]
 $script:ui.PageHint.Text=@{Themes='Choose one face or build a rotation from your favourites.';Editor='Combine artwork and live sensor elements on a 320 × 240 canvas.';Settings='Playback, device controls and diagnostics in one place.'}[$page]}
function Select-Face($face){$script:selectedFace=$face;$script:ui.SelectedName.Text=$face.name;$script:ui.ThemePreview.Source=Bitmap $face.path;$script:ui.UseSingle.IsEnabled=$true;$script:ui.EditSelected.IsEnabled=$true}
function Render-Cards {
 $script:ui.ThemeCards.Children.Clear();$query=$script:ui.SearchThemes.Text.Trim()
 foreach($face in $script:faces){if($query -and $face.name -notlike "*$query*" -and $face.id -notlike "*$query*"){continue}
  $card=New-Object Windows.Controls.Border;$card.Width=174;$card.Margin='0,0,10,10';$card.Padding=8;$card.CornerRadius=10;$card.Background=$script:window.FindResource('PanelBrush');$card.BorderThickness=1;$card.BorderBrush=$script:window.FindResource('LineBrush')
  $stack=New-Object Windows.Controls.StackPanel;$card.Child=$stack
  $choose=New-Object Windows.Controls.Button;$choose.Padding=0;$choose.Margin=0;$choose.Height=114;$choose.ToolTip=$face.name;$image=New-Object Windows.Controls.Image;$image.Source=Bitmap $face.path;$image.Stretch='UniformToFill';$choose.Content=$image
  $captured=$face;$choose.Add_Click({Safe-Action {Select-Face $captured}}.GetNewClosure());$stack.Children.Add($choose)|Out-Null
  $name=New-Object Windows.Controls.TextBlock;$name.Text=$face.name;$name.Margin='0,8,0,4';$name.FontWeight='SemiBold';$name.TextTrimming='CharacterEllipsis';$name.TextWrapping='NoWrap';$name.ToolTip=$face.name;$stack.Children.Add($name)|Out-Null
  $check=New-Object Windows.Controls.CheckBox;$check.Content='In playlist';$check.IsChecked=$script:checked.Contains($face.id);$check.Tag=$face.id
  $check.Add_Checked({$script:checked.Add([string]$this.Tag)|Out-Null;Update-Checks});$check.Add_Unchecked({$script:checked.Remove([string]$this.Tag)|Out-Null;Update-Checks});$stack.Children.Add($check)|Out-Null
  $script:ui.ThemeCards.Children.Add($card)|Out-Null
 }
 $script:ui.ThemeCount.Text="$($script:faces.Count) faces"
 if($script:ui.ThemeCards.Children.Count -eq 0){$empty=New-Object Windows.Controls.TextBlock;$empty.Text='No faces match your search.';$empty.Margin=20;$script:ui.ThemeCards.Children.Add($empty)|Out-Null}
}
function Update-Checks {$script:ui.PlaylistHint.Text=if($script:checked.Count){"$($script:checked.Count) checked faces · plays in library order."}else{'No checks = rotate every available face.'}}
function Load-Faces {
 Notice 'Loading theme previews…'
 & $script:binary 2>> (Join-Path $script:root 'studio-error.log')|Out-Null
 if($LASTEXITCODE -ne 0){throw 'Renderer could not create previews. See studio-error.log.'}
 $script:faces=@()
 foreach($line in (& $script:binary --list)){if($line -match '^([^:]+): (.+)$'){$path=Join-Path $script:root ('preview/'+$Matches[1]+'.png');if(Test-Path $path){$script:faces+=@{id=$Matches[1];name=$Matches[2];path=$path}}}}
 if(-not $script:faces.Count){throw 'No renderable themes found.'}
 Render-Cards
 $face=$script:faces|Where-Object {$_.id -eq $script:settings.face}|Select-Object -First 1;if(-not $face){$face=$script:faces[0]};Select-Face $face;Update-Checks;Notice 'Ready · previews are demo; LCD readings are live.'
}
function Set-Playback($rotate){if(-not $script:selectedFace){throw 'Select a theme first.'};$script:settings.face=$script:selectedFace.id;$script:settings.rotate=$rotate;$script:settings.dwell=Dwell $script:ui.ThemeDwell.Text;$script:settings.playlist=@($script:faces|Where-Object {$script:checked.Contains($_.id)}|ForEach-Object {$_.id});$script:ui.SettingsDwell.Text=[string]$script:settings.dwell;Persist-Playback;Notice $(if($rotate){'Rotation saved and applied to the display.'}else{'Single face saved and applied to the display.'})}
function Element-Valid($e) {return ($e.x -ge 0 -and $e.y -ge 0 -and $e.width -ge 12 -and $e.height -ge 12 -and $e.x+$e.width -le 320 -and $e.y+$e.height -le 240 -and $e.size -ge 8 -and $e.size -le 72 -and $e.max -gt 0)}
function Sensor($id){return ($script:sensors|Where-Object {$_.id -eq $id}|Select-Object -First 1)}
function Brush($rgb){return New-Object Windows.Media.SolidColorBrush([Windows.Media.Color]::FromRgb($rgb[0],$rgb[1],$rgb[2]))}
function Text-Visual($text,$size,$colour){$t=New-Object Windows.Controls.TextBlock;$t.Text=$text;$t.FontSize=$size;$t.FontWeight='SemiBold';$t.Foreground=$colour;$t.TextWrapping='NoWrap';return $t}
function Element-Visual($e){
 $panel=New-Object Windows.Controls.Canvas;$panel.Width=$e.width;$panel.Height=$e.height;$panel.ClipToBounds=$true;$sensor=Sensor $e.sensor;$accent=Brush $e.color
 $label=$e.sensor.ToUpper().Replace('.',' ');$value=if($sensor){[string]$sensor.demo+$sensor.unit}else{'—'}
 if($e.kind -eq 'ring'){
  $radius=($e.width/2)-6;$radius=[Math]::Min($radius,($e.height/2)-6);$cx=$e.width/2;$cy=$e.height/2
  foreach($spec in @(@{fraction=1;color=[Windows.Media.Brushes]::DarkSlateGray},@{fraction=[Math]::Max(0.001,[Math]::Min(1,$sensor.demo/$e.max));color=$accent})){
   $start=135*[Math]::PI/180;$end=(135+270*$spec.fraction)*[Math]::PI/180
   $fig=New-Object Windows.Media.PathFigure;$fig.StartPoint=New-Object Windows.Point(($cx+$radius*[Math]::Cos($start)),($cy+$radius*[Math]::Sin($start)))
   $seg=New-Object Windows.Media.ArcSegment;$seg.Point=New-Object Windows.Point(($cx+$radius*[Math]::Cos($end)),($cy+$radius*[Math]::Sin($end)));$seg.Size=New-Object Windows.Size($radius,$radius);$seg.IsLargeArc=($spec.fraction*270 -gt 180);$seg.SweepDirection='Clockwise';$fig.Segments.Add($seg)
   $geo=New-Object Windows.Media.PathGeometry;$geo.Figures.Add($fig);$path=New-Object Windows.Shapes.Path;$path.Data=$geo;$path.Stroke=$spec.color;$path.StrokeThickness=5;$path.StrokeStartLineCap='Round';$path.StrokeEndLineCap='Round';$panel.Children.Add($path)|Out-Null
  }
  $small=Text-Visual $label 10 $accent;$small.Width=$e.width;$small.TextAlignment='Center';[Windows.Controls.Canvas]::SetTop($small,$cy-22);$panel.Children.Add($small)|Out-Null
  $big=Text-Visual $value $e.size ([Windows.Media.Brushes]::White);$big.Width=$e.width;$big.TextAlignment='Center';[Windows.Controls.Canvas]::SetTop($big,$cy-5);$panel.Children.Add($big)|Out-Null
 }elseif($e.kind -eq 'bar'){
  $text=Text-Visual ($label+' '+$value) $e.size $accent;$panel.Children.Add($text)|Out-Null
  $track=New-Object Windows.Shapes.Rectangle;$track.Width=$e.width;$track.Height=[Math]::Max(4,$e.height-$e.size-4);$track.RadiusX=4;$track.RadiusY=4;$track.Fill=[Windows.Media.Brushes]::DarkSlateGray;[Windows.Controls.Canvas]::SetTop($track,$e.size+4);$panel.Children.Add($track)|Out-Null
  $fill=New-Object Windows.Shapes.Rectangle;$fill.Width=$e.width*[Math]::Max(0,[Math]::Min(1,$sensor.demo/$e.max));$fill.Height=$track.Height;$fill.RadiusX=4;$fill.RadiusY=4;$fill.Fill=$accent;[Windows.Controls.Canvas]::SetTop($fill,$e.size+4);$panel.Children.Add($fill)|Out-Null
 }else{
  $small=Text-Visual $label ([Math]::Min(18,$e.size)) $accent;$panel.Children.Add($small)|Out-Null
  $big=Text-Visual $value $e.size ([Windows.Media.Brushes]::White);[Windows.Controls.Canvas]::SetTop($big,[Math]::Min(18,$e.size)+2);$panel.Children.Add($big)|Out-Null
 }
 return $panel
}
function Update-Inspector {
 $script:updating=$true
 if($script:selectedElement -ge 0 -and $script:selectedElement -lt $script:elements.Count){$e=$script:elements[$script:selectedElement];foreach($pair in @(@('PropX','x'),@('PropY','y'),@('PropWidth','width'),@('PropHeight','height'),@('PropSize','size'),@('PropMax','max'))){$script:ui[$pair[0]].Text=[string]$e[$pair[1]]};$script:ui.SensorSelect.SelectedValue=$e.sensor}
 $script:ui.ApplyProperties.IsEnabled=($script:selectedElement -ge 0);$script:ui.DeleteLayer.IsEnabled=($script:selectedElement -ge 0)
 $script:updating=$false
}
function Draw-Design([switch]$PreserveInspector) {
 $script:ui.DesignCanvas.Children.Clear()
 if($script:background){$img=New-Object Windows.Controls.Image;$img.Source=$script:background;$img.Width=320;$img.Height=240;$img.Stretch='Fill';$img.IsHitTestVisible=$false;$script:ui.DesignCanvas.Children.Add($img)|Out-Null}
 $script:updating=$true;$script:ui.Layers.Items.Clear()
 for($i=0;$i -lt $script:elements.Count;$i++){
  $e=$script:elements[$i];$visual=Element-Visual $e;$border=New-Object Windows.Controls.Border;$border.Child=$visual;$border.Width=$e.width;$border.Height=$e.height;$border.Background=[Windows.Media.Brushes]::Transparent;$border.Tag=$i
  if($i -eq $script:selectedElement){$border.BorderBrush=[Windows.Media.Brushes]::White;$border.BorderThickness=1}
  [Windows.Controls.Canvas]::SetLeft($border,$e.x);[Windows.Controls.Canvas]::SetTop($border,$e.y)
  $border.Add_MouseLeftButtonDown({param($sender,$eventArgs)$script:selectedElement=[int]$sender.Tag;$script:dragStart=$eventArgs.GetPosition($script:ui.DesignCanvas);$entry=$script:elements[$script:selectedElement];$script:dragX=$entry.x;$script:dragY=$entry.y;$script:dragWidth=$entry.width;$script:dragHeight=$entry.height;$relative=$eventArgs.GetPosition($sender);$script:resizing=($relative.X -gt $entry.width-12 -and $relative.Y -gt $entry.height-12);$script:dragging=$true;$sender.CaptureMouse()|Out-Null;$script:ui.Layers.SelectedIndex=$script:selectedElement;Update-Inspector;$eventArgs.Handled=$true})
  $border.Add_MouseMove({param($sender,$eventArgs)if($script:dragging -and $sender.IsMouseCaptured){$pos=$eventArgs.GetPosition($script:ui.DesignCanvas);$entry=$script:elements[$script:selectedElement];if($script:resizing){$entry.width=[Math]::Round([Math]::Max(12,[Math]::Min(320-$entry.x,$script:dragWidth+$pos.X-$script:dragStart.X)));$entry.height=[Math]::Round([Math]::Max(12,[Math]::Min(240-$entry.y,$script:dragHeight+$pos.Y-$script:dragStart.Y)));$sender.Width=$entry.width;$sender.Height=$entry.height}else{$entry.x=[Math]::Round([Math]::Max(0,[Math]::Min(320-$entry.width,$script:dragX+$pos.X-$script:dragStart.X)));$entry.y=[Math]::Round([Math]::Max(0,[Math]::Min(240-$entry.height,$script:dragY+$pos.Y-$script:dragStart.Y)));[Windows.Controls.Canvas]::SetLeft($sender,$entry.x);[Windows.Controls.Canvas]::SetTop($sender,$entry.y)};$script:dirty=$true;Update-Inspector}})
  $border.Add_MouseLeftButtonUp({param($sender,$eventArgs)$script:dragging=$false;$sender.ReleaseMouseCapture();Draw-Design;$eventArgs.Handled=$true})
  $script:ui.DesignCanvas.Children.Add($border)|Out-Null
  if($i -eq $script:selectedElement){$handle=New-Object Windows.Shapes.Rectangle;$handle.Width=7;$handle.Height=7;$handle.Fill=[Windows.Media.Brushes]::White;$handle.IsHitTestVisible=$false;[Windows.Controls.Canvas]::SetLeft($handle,$e.x+$e.width-7);[Windows.Controls.Canvas]::SetTop($handle,$e.y+$e.height-7);$script:ui.DesignCanvas.Children.Add($handle)|Out-Null}
  $script:ui.Layers.Items.Add(($e.kind+' · '+$(if($e.kind -eq 'label'){$e.text}elseif($e.kind -eq 'panel'){'Background'}else{(Sensor $e.sensor).label})))|Out-Null
 }
 $script:ui.Layers.SelectedIndex=$script:selectedElement;$script:updating=$false;if(-not $PreserveInspector){Update-Inspector};$script:ui.RecropImage.IsEnabled=($null -ne $script:sourceImage)
}
function Add-Element($kind){$sensor=$script:ui.SensorSelect.SelectedItem;$e=@{kind=$kind;sensor=$sensor.id;x=20;y=65;width=110;height=110;size=28;max=$sensor.max;color=@(0,195,255)};if($kind -eq 'bar'){$e.width=190;$e.height=55;$e.size=16};if($kind -eq 'text'){$e.height=65};$script:elements.Add($e)|Out-Null;$script:selectedElement=$script:elements.Count-1;$script:dirty=$true;Draw-Design;Notice 'Element added. Drag it or change its properties.'}
function Apply-Properties {
 if($script:selectedElement -lt 0){throw 'Select a layer first.'};$e=$script:elements[$script:selectedElement].Clone()
 foreach($pair in @(@('PropX','x'),@('PropY','y'),@('PropWidth','width'),@('PropHeight','height'),@('PropSize','size'),@('PropMax','max'))){$n=0.0;if(-not [double]::TryParse($script:ui[$pair[0]].Text,[ref]$n) -or [double]::IsNaN($n) -or [double]::IsInfinity($n)){throw 'Use valid numbers in the inspector.'};$e[$pair[1]]=$n}
 if($e.size -ne [Math]::Floor($e.size)){throw 'Font size must be a whole number.'};$e.sensor=$script:ui.SensorSelect.SelectedValue;if(-not(Element-Valid $e)){throw 'Keep elements inside 320 × 240. Size 8–72, width/height at least 12, and maximum above zero.'}
 $script:elements[$script:selectedElement]=$e;$script:dirty=$true;Draw-Design;Notice 'Element updated.'
}
function Confirm-Discard {if(-not $script:dirty){return $true};return ([Windows.MessageBox]::Show('Discard unsaved changes?','Unsaved design','YesNo','Question') -eq 'Yes')}
function New-Design {if(-not(Confirm-Discard)){return};$script:elements.Clear();$script:selectedElement=-1;$script:background=$null;$script:sourceImage=$null;$script:dirty=$false;$script:ui.DesignName.Text='My custom face';$script:ui.DesignId.Text='my-custom-face';Draw-Design}
. (Join-Path $PSScriptRoot 'studio-hud.ps1')
. (Join-Path $PSScriptRoot 'studio-actions.ps1')
