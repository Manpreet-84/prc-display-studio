function Crop-Image($source) {
 [xml]$xml=@'
<Window xmlns="http://schemas.microsoft.com/winfx/2006/xaml/presentation" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml" Title="Crop background" Width="720" Height="640" MinWidth="540" MinHeight="470" WindowStartupLocation="CenterOwner" Background="#F4F5F8">
 <Grid Margin="20"><Grid.RowDefinitions><RowDefinition Height="Auto"/><RowDefinition Height="*"/><RowDefinition Height="Auto"/></Grid.RowDefinitions><TextBlock Text="Drag to position · zoom to crop · fixed 4:3 LCD shape" Foreground="#445572" Margin="0,0,0,15"/><Viewbox Grid.Row="1" Stretch="Uniform"><Border Background="#080C14"><Image x:Name="CropImage" Width="640" Height="480" Stretch="Fill" Cursor="SizeAll"/></Border></Viewbox><StackPanel Grid.Row="2" Margin="0,15,0,0"><TextBlock Text="Zoom"/><Slider x:Name="CropZoom" Minimum="1" Maximum="5" Value="1" Margin="0,8"/><StackPanel Orientation="Horizontal" HorizontalAlignment="Right"><Button x:Name="CancelCrop" Content="Cancel" Padding="20,8" Margin="0,0,10,0"/><Button x:Name="AcceptCrop" Content="Use crop" Padding="20,8" Background="#007AFF" Foreground="White"/></StackPanel></StackPanel></Grid>
</Window>
'@
 $r=New-Object Xml.XmlNodeReader($xml);$script:cropWindow=[Windows.Markup.XamlReader]::Load($r);$script:cropWindow.Owner=$script:window
 $script:cropView=$script:cropWindow.FindName('CropImage');$script:cropZoom=$script:cropWindow.FindName('CropZoom');$script:cropSource=$source;$script:cropCx=$source.PixelWidth/2;$script:cropCy=$source.PixelHeight/2;$script:cropDragging=$false
 $script:cropUpdate={
  $script:cropWidth=[Math]::Min($script:cropSource.PixelWidth,$script:cropSource.PixelHeight*4/3)/$script:cropZoom.Value
  $script:cropHeight=$script:cropWidth*3/4
  $script:cropCx=[Math]::Max($script:cropWidth/2,[Math]::Min($script:cropSource.PixelWidth-$script:cropWidth/2,$script:cropCx));$script:cropCy=[Math]::Max($script:cropHeight/2,[Math]::Min($script:cropSource.PixelHeight-$script:cropHeight/2,$script:cropCy))
  $rect=New-Object Windows.Int32Rect([int][Math]::Floor($script:cropCx-$script:cropWidth/2),[int][Math]::Floor($script:cropCy-$script:cropHeight/2),[int][Math]::Max(1,[Math]::Floor($script:cropWidth)),[int][Math]::Max(1,[Math]::Floor($script:cropHeight)))
  $script:cropResult=New-Object Windows.Media.Imaging.CroppedBitmap($script:cropSource,$rect);$script:cropView.Source=$script:cropResult
 }
 $script:cropZoom.Add_ValueChanged({& $script:cropUpdate})
 $script:cropView.Add_MouseLeftButtonDown({param($s,$e)$script:cropDragging=$true;$script:cropLast=$e.GetPosition($s);$s.CaptureMouse()|Out-Null})
 $script:cropView.Add_MouseMove({param($s,$e)if($script:cropDragging){$p=$e.GetPosition($s);$script:cropCx-=($p.X-$script:cropLast.X)*$script:cropWidth/640;$script:cropCy-=($p.Y-$script:cropLast.Y)*$script:cropHeight/480;$script:cropLast=$p;& $script:cropUpdate}})
 $script:cropView.Add_MouseLeftButtonUp({param($s,$e)$script:cropDragging=$false;$s.ReleaseMouseCapture()})
 $script:cropWindow.FindName('CancelCrop').Add_Click({$script:cropWindow.DialogResult=$false})
 $script:cropWindow.FindName('AcceptCrop').Add_Click({$script:cropWindow.DialogResult=$true})
 & $script:cropUpdate
 if($script:cropWindow.ShowDialog()){$result=$script:cropResult.Clone();$result.Freeze();return $result};return $null
}
function Import-Image {
 $dialog=New-Object Microsoft.Win32.OpenFileDialog;$dialog.Filter='Images|*.png;*.jpg;*.jpeg;*.bmp;*.webp'
 if($dialog.ShowDialog($script:window)){
  $file=$dialog.FileName
  if([IO.Path]::GetExtension($file) -eq '.webp'){$temp=Join-Path $script:root 'preview/import-image.png';& $script:binary --decode-image $file $temp|Out-Null;if($LASTEXITCODE -ne 0){throw 'Cannot decode image.'};$file=$temp}
  $source=Bitmap $file
  if($source.PixelWidth -lt 4 -or $source.PixelHeight -lt 3){throw 'Image is too small.'}
  $cropped=Crop-Image $source
  if($cropped){$script:sourceImage=$source;$script:background=$cropped;$script:dirty=$true;Draw-Design;Notice 'Background added. Add HUD elements from the palette.'}
 }
}
function Save-Bitmap($source,$path) {
 $visual=New-Object Windows.Media.DrawingVisual;$dc=$visual.RenderOpen();$dc.DrawRectangle([Windows.Media.Brushes]::Black,$null,(New-Object Windows.Rect(0,0,320,240)));if($source){$dc.DrawImage($source,(New-Object Windows.Rect(0,0,320,240)))};$dc.Close()
 $bitmap=New-Object Windows.Media.Imaging.RenderTargetBitmap(320,240,96,96,[Windows.Media.PixelFormats]::Pbgra32);$bitmap.Render($visual)
 $enc=New-Object Windows.Media.Imaging.PngBitmapEncoder;$enc.Frames.Add([Windows.Media.Imaging.BitmapFrame]::Create($bitmap));$stream=[IO.File]::Create($path);try{$enc.Save($stream)}finally{$stream.Dispose()}
}
function Save-Design {
 if($script:selectedElement -ge 0){Apply-Properties}
 $id=$script:ui.DesignId.Text.Trim();$name=$script:ui.DesignName.Text.Trim()
 if($id -notmatch '^[a-z0-9][a-z0-9-]{0,63}$'){throw 'Theme ID: lowercase letters, numbers and hyphens; maximum 64 characters.'}
 if(-not $name -or $name.Length -gt 128){throw 'Enter a theme name, maximum 128 characters.'}
 if($script:elements.Count -eq 0){throw 'Add at least one HUD element.'}
 if($script:elements.Count -gt 64){throw 'A face can contain up to 64 elements.'}
 foreach($e in $script:elements){if(-not(Element-Valid $e)){throw 'An element extends outside the canvas. Move or resize it first.'}}
 $dir=Join-Path $env:PRC_THEME_DIR $id
 if(($script:faces|Where-Object {$_.id -eq $id}) -and -not(Test-Path (Join-Path $dir 'theme.json'))){throw 'This ID belongs to a built-in face. Use a new ID.'}
 if(Test-Path (Join-Path $dir 'theme.json')){if([Windows.MessageBox]::Show('Replace the existing theme?','Save theme','YesNo','Question') -ne 'Yes'){return}}
 $manifest=@{name=$name;background='art.png';elements=@($script:elements.ToArray())}|ConvertTo-Json -Depth 8 -Compress
 if([Text.Encoding]::UTF8.GetByteCount($manifest) -gt 65536){throw 'Too many elements; theme manifest exceeds 64 KB.'}
 New-Item -ItemType Directory -Path $dir -Force|Out-Null
 Save-Bitmap $script:background (Join-Path $dir 'art.png')
 [IO.File]::WriteAllText((Join-Path $dir 'theme.json.tmp'),$manifest);Move-Item -LiteralPath (Join-Path $dir 'theme.json.tmp') -Destination (Join-Path $dir 'theme.json') -Force
 $script:dirty=$false;Load-Faces;Notice 'Theme saved. Click Reload themes in Settings to add it to the running display.'
}
function Edit-Selected {
 if(-not $script:selectedFace){throw 'Select a face first.'};if(-not(Confirm-Discard)){return}
 $dir=Join-Path $env:PRC_THEME_DIR $script:selectedFace.id;$script:elements.Clear();$script:selectedElement=-1
 if(Test-Path (Join-Path $dir 'theme.json')){
  $config=Get-Content (Join-Path $dir 'theme.json') -Raw|ConvertFrom-Json
  $art=Join-Path $dir $config.background
  if([IO.Path]::GetExtension($art) -eq '.webp'){$temp=Join-Path $script:root 'preview/edit-background.png';& $script:binary --decode-image $art $temp|Out-Null;if($LASTEXITCODE -ne 0){throw 'Cannot open background.'};$art=$temp}
  $script:background=Bitmap $art;$script:sourceImage=$script:background
  foreach($element in $config.elements){$e=@{};foreach($prop in $element.PSObject.Properties){$e[$prop.Name]=$prop.Value};$script:elements.Add($e)|Out-Null}
  $script:ui.DesignId.Text=$script:selectedFace.id;$script:ui.DesignName.Text=$config.name
 }else{
  $clean=Join-Path $script:root ('faces/sheet/'+$script:selectedFace.id+'-base.webp')
  if(Test-Path $clean){$temp=Join-Path $script:root 'preview/edit-background.png';& $script:binary --decode-image $clean $temp|Out-Null;$script:background=Bitmap $temp;$script:sourceImage=$script:background}else{$script:background=$null;$script:sourceImage=$null}
  $script:ui.DesignId.Text=$script:selectedFace.id+'-custom';$script:ui.DesignName.Text=$script:selectedFace.name+' custom'
 }
 $script:dirty=$true;Draw-Design;Navigate 'Editor';Notice 'Artwork loaded. Add or edit live HUD layers.'
}
function Send-Command($text){$path=Join-Path $script:root 'control-command.txt';$tmp=$path+'.tmp';[IO.File]::WriteAllText($tmp,$text);Move-Item -LiteralPath $tmp -Destination $path -Force}
function Start-Display {Start-Process powershell.exe -ArgumentList ('-NoProfile -WindowStyle Hidden -File "'+(Join-Path $PSScriptRoot 'start-autostart.ps1')+'"') -WindowStyle Hidden;Notice 'Starting display; USB detection may take a moment.'}
function Restart-Display {Send-Command 'quit';Start-Process powershell.exe -ArgumentList ('-NoProfile -WindowStyle Hidden -Command "Start-Sleep -Seconds 3; & '''+(Join-Path $PSScriptRoot 'start-autostart.ps1')+'''"') -WindowStyle Hidden;Notice 'Reloading themes on the display…'}
$script:ui.NavThemes.Add_Click({Navigate 'Themes'});$script:ui.NavEditor.Add_Click({Navigate 'Editor'});$script:ui.NavSettings.Add_Click({Navigate 'Settings'})
$script:ui.RefreshThemes.Add_Click({Safe-Action {Load-Faces}});$script:ui.SearchThemes.Add_TextChanged({Render-Cards})
$script:ui.UseSingle.Add_Click({Safe-Action {Set-Playback $false}});$script:ui.PlayPlaylist.Add_Click({Safe-Action {Set-Playback $true}})
$script:ui.CheckAll.Add_Click({foreach($face in $script:faces){$script:checked.Add($face.id)|Out-Null};Render-Cards;Update-Checks});$script:ui.UncheckAll.Add_Click({$script:checked.Clear();Render-Cards;Update-Checks})
$script:ui.EditSelected.Add_Click({Safe-Action {Edit-Selected}})
$script:ui.ImportImage.Add_Click({Safe-Action {Import-Image}})
$script:ui.RecropImage.Add_Click({Safe-Action {if($script:sourceImage){$crop=Crop-Image $script:sourceImage;if($crop){$script:background=$crop;$script:dirty=$true;Draw-Design}}}})
$script:ui.ClearImage.Add_Click({$script:background=$null;$script:sourceImage=$null;$script:dirty=$true;Draw-Design})
$script:ui.NewDesign.Add_Click({New-Design});$script:ui.SaveDesign.Add_Click({Safe-Action {Save-Design}})
$script:ui.Layers.Add_SelectionChanged({if(-not $script:updating){$script:selectedElement=$script:ui.Layers.SelectedIndex;Draw-Design}})
$script:ui.ApplyProperties.Add_Click({Safe-Action {Apply-Properties}})
$script:ui.SensorSelect.Add_SelectionChanged({param($sender,$change)if(-not $script:updating -and $script:selectedElement -ge 0 -and $change.AddedItems.Count -gt 0){$script:elements[$script:selectedElement].sensor=$change.AddedItems[0].id;$script:elements[$script:selectedElement].max=$change.AddedItems[0].max;$script:ui.PropMax.Text=[string]$change.AddedItems[0].max;$script:dirty=$true;Draw-Design -PreserveInspector}})
$script:ui.DeleteLayer.Add_Click({if($script:selectedElement -ge 0){$script:elements.RemoveAt($script:selectedElement);$script:selectedElement=[Math]::Min($script:selectedElement,$script:elements.Count-1);$script:dirty=$true;Draw-Design}})
$script:ui.LayerUp.Add_Click({$i=$script:selectedElement;if($i -gt 0){$temp=$script:elements[$i-1];$script:elements[$i-1]=$script:elements[$i];$script:elements[$i]=$temp;$script:selectedElement--;$script:dirty=$true;Draw-Design}})
$script:ui.LayerDown.Add_Click({$i=$script:selectedElement;if($i -ge 0 -and $i -lt $script:elements.Count-1){$temp=$script:elements[$i+1];$script:elements[$i+1]=$script:elements[$i];$script:elements[$i]=$temp;$script:selectedElement++;$script:dirty=$true;Draw-Design}})
$script:ui.SaveSettings.Add_Click({Safe-Action {$script:settings.dwell=Dwell $script:ui.SettingsDwell.Text;$script:ui.ThemeDwell.Text=[string]$script:settings.dwell;Persist-Playback;Notice 'Playback timing saved.'}})
$script:ui.StartDisplay.Add_Click({Start-Display});$script:ui.StopDisplay.Add_Click({Send-Command 'quit';Notice 'Stopping display.'});$script:ui.ReloadDisplay.Add_Click({Restart-Display})
$script:ui.OpenThemesFolder.Add_Click({Start-Process explorer.exe -ArgumentList $env:PRC_THEME_DIR});$script:ui.OpenLogs.Add_Click({Start-Process notepad.exe -ArgumentList (Join-Path $script:root 'display-live.log')})
function Render-Element-Palette {
 $script:ui.ElementPalette.Children.Clear()
 $sensor=$script:ui.AddSensor.SelectedItem
 if(-not $sensor){return}
 foreach($kind in @('text','bar','ring','graph','label','panel')){
  $e=@{kind=$kind;sensor=$sensor.id;x=0;y=0;width=120;height=70;size=22;max=$sensor.max;color=@(0,195,255)}
  $e=Normalize-Element $e;$e.opacity=0;$e.show_label=$false;$e.text='MY PC';$e.label_size=10
  if($kind -in @('bar','graph')){$e.size=16};if($kind -eq 'panel'){$e.opacity=150}
  $button=New-Object Windows.Controls.Button;$button.Width=142;$button.Height=98;$button.Padding=5;$button.Background=New-Object Windows.Media.SolidColorBrush([Windows.Media.Color]::FromRgb(12,18,30))
  $panel=New-Object Windows.Controls.StackPanel;$panel.Children.Add((Element-Visual $e))|Out-Null
  $title=@{text='Number';bar='Bar';ring='Circular gauge';graph='Line graph';label='Custom text';panel='Background panel'}[$kind]
  $label=Text-Visual $title 11 ([Windows.Media.Brushes]::White);$label.TextAlignment='Center';$panel.Children.Add($label)|Out-Null;$button.Content=$panel
  $type=$kind;$button.Add_Click({Safe-Action {Add-Element $type}}.GetNewClosure());$script:ui.ElementPalette.Children.Add($button)|Out-Null
 }
}
foreach($sensor in $script:sensors){$script:ui.AddSensor.Items.Add([pscustomobject]$sensor)|Out-Null}
$script:ui.AddSensor.Add_SelectionChanged({param($sender,$change)if($change.AddedItems.Count -gt 0){$script:ui.AddSensor.Dispatcher.BeginInvoke([Action]{Render-Element-Palette})|Out-Null}})
$script:ui.AddSensor.SelectedIndex=0
Render-Element-Palette
foreach($rgb in @(@(0,195,255),@(255,77,83),@(64,219,137),@(190,119,255),@(255,194,66),@(245,248,255))){$button=New-Object Windows.Controls.Button;$button.Width=32;$button.Height=32;$button.Padding=0;$button.Background=Brush $rgb;$button.Tag=$rgb;$button.ToolTip='Accent colour';$button.Add_Click({if($script:selectedElement -ge 0){Apply-Colour @($this.Tag)}});$script:ui.ColourPalette.Children.Add($button)|Out-Null}
foreach($mode in @(@{id='text';label='Sensor number'},@{id='bar';label='Horizontal bar'},@{id='ring';label='Circular gauge'},@{id='graph';label='Line graph'},@{id='label';label='Custom text'},@{id='panel';label='Background panel'})){$script:ui.ModeSelect.Items.Add([pscustomobject]$mode)|Out-Null}
foreach($target in @(@{id='color';label='Bar / gauge / graph'},@{id='value_color';label='Value text'},@{id='label_color';label='Label text'},@{id='track_color';label='Track / grid'},@{id='background_color';label='Background panel'})){$script:ui.ColourTarget.Items.Add([pscustomobject]$target)|Out-Null}
$script:ui.ColourTarget.SelectedIndex=0
$script:ui.ColourTarget.Add_SelectionChanged({param($sender,$change)if(-not $script:updating -and $change.AddedItems.Count -gt 0){Update-Colour $change.AddedItems[0].id}})
$script:ui.ApplyColour.Add_Click({Safe-Action {$hex=$script:ui.ColourHex.Text.Trim();if($hex -notmatch '^#[0-9a-fA-F]{6}$'){throw 'Use #RRGGBB, for example #00C3FF.'};$rgb=@([Convert]::ToInt32($hex.Substring(1,2),16),[Convert]::ToInt32($hex.Substring(3,2),16),[Convert]::ToInt32($hex.Substring(5,2),16));Apply-Colour $rgb}})
$script:ui.PropOpacity.Add_ValueChanged({$script:ui.OpacityReadout.Text=([Math]::Round($script:ui.PropOpacity.Value).ToString()+'%');if(-not $script:updating -and $script:selectedElement -ge 0){$script:elements[$script:selectedElement].opacity=[int][Math]::Round($script:ui.PropOpacity.Value*255/100);$script:dirty=$true;Draw-Design -PreserveInspector}})
$script:ui.ModeSelect.Add_SelectionChanged({param($sender,$change)if(-not $script:updating -and $script:selectedElement -ge 0 -and $change.AddedItems.Count -gt 0){$script:elements[$script:selectedElement].kind=$change.AddedItems[0].id;$script:dirty=$true;Draw-Design -PreserveInspector}})
$script:timer=New-Object Windows.Threading.DispatcherTimer;$script:timer.Interval=[TimeSpan]::FromSeconds(5);$script:timer.Add_Tick({$running=Get-Process prc-lcd -ErrorAction SilentlyContinue;$script:ui.DeviceState.Text=if($running){'Display app running'}else{'Display app stopped'}})
$script:window.Add_Closing({param($s,$e)if(-not(Confirm-Discard)){$e.Cancel=$true}})
Navigate 'Themes';Draw-Design;Update-Playback
if($SelfTest){
 Load-Faces
 if($script:faces.Count -lt 42){throw 'Theme library did not load.'}
 if(-not $script:ui.ThemePreview.Source){throw 'No preview selected.'}
 $click=New-Object Windows.RoutedEventArgs([Windows.Controls.Button]::ClickEvent)
 $script:ui.ThemeCards.Children[0].Child.Children[0].RaiseEvent($click)
 if($script:selectedFace.id -ne $script:faces[0].id){throw 'Theme card selection failed.'}
 $script:ui.SearchThemes.Text='no-such-theme-qa';if($script:ui.ThemeCards.Children.Count -ne 1){throw 'Empty search state failed.'};$script:ui.SearchThemes.Text=''
 foreach($bad in @('0','1','3601','abc','2.5')){try{Dwell $bad|Out-Null;throw 'Bad timing accepted'}catch{if($_.Exception.Message -eq 'Bad timing accepted'){throw}}}

 $event=New-Object Windows.RoutedEventArgs([Windows.Controls.Button]::ClickEvent)
 $script:ui.ElementPalette.Children[1].RaiseEvent($event)
 if($script:elements.Count -ne 1 -or $script:elements[0].kind -ne 'bar'){throw 'Visual palette click failed.'}
 if($script:elements[0].show_label){throw 'Sensor HUD must start without a label.'}
 if($script:elements[0].opacity -ne 0){throw 'Sensor HUD must start transparent.'}
 $script:elements[0].x=20;$script:elements[0].y=160;$script:elements[0].height=72;Add-Element 'ring';$script:elements[1].x=170;$script:elements[1].y=55;Add-Element 'text';$script:elements[2].x=20;$script:elements[2].y=55;Draw-Design
 if($script:elements.Count -ne 3){throw 'Palette failed.'}
 # Sensor-first add flow, and switching display mode must retain the chosen reading.
 $script:ui.AddSensor.SelectedValue='cpu.power';Render-Element-Palette
 $script:ui.ElementPalette.Children[2].RaiseEvent((New-Object Windows.RoutedEventArgs([Windows.Controls.Button]::ClickEvent)))
 if($script:elements[3].sensor -ne 'cpu.power' -or $script:elements[3].kind -ne 'ring' -or $script:elements[3].max -ne 250){throw 'Sensor-first gauge failed.'}
 $script:ui.ModeSelect.SelectedValue='graph'
 if($script:elements[3].sensor -ne 'cpu.power' -or $script:elements[3].kind -ne 'graph'){throw 'Changing mode changed the sensor.'}
 $script:elements.RemoveAt(3);$script:selectedElement=2;$script:ui.AddSensor.SelectedValue='cpu.temp';Render-Element-Palette;Draw-Design
 # Inspector keeps text, units, mode, opacity and per-part colours in the manifest.
 $script:ui.ShowLabel.IsChecked=$true;$script:ui.PropLabel.Text='Processor';$script:ui.PropUnit.Text=' C';$script:ui.PropDecimals.Text='1';$script:ui.PropOpacity.Value=80
 Apply-Properties
 if($script:elements[2].label -ne 'Processor' -or $script:elements[2].unit -ne ' C' -or $script:elements[2].opacity -ne 204){throw 'HUD content/style settings did not apply.'}
 $script:ui.ColourTarget.SelectedValue='value_color';Apply-Colour @(255,194,66)
 if($script:elements[2].value_color[1] -ne 194){throw 'Value colour did not apply.'}
 $script:ui.ModeSelect.SelectedValue='graph'
 if($script:elements[2].kind -ne 'graph'){throw ('Mode switching failed: selected='+$script:selectedElement+' mode='+$script:ui.ModeSelect.SelectedValue+' kind='+$script:elements[2].kind)}
 $script:ui.ModeSelect.SelectedValue='text'
 # Graph/static text/panel modes render through the same native saved format.
 foreach($kind in @('graph','label','panel')){$extra=(Normalize-Element $script:elements[0]).Clone();$extra.kind=$kind;$extra.x=5;$extra.y=5;$extra.width=150;$extra.height=95;$extra.text='MY PC';$script:elements.Add($extra)|Out-Null}
 $extraCount=$script:elements.Count
 Draw-Design
 $script:ui.PropWidth.Text='500';try{Apply-Properties;throw 'Invalid extents accepted'}catch{if($_.Exception.Message -eq 'Invalid extents accepted'){throw}}
 # Verify custom manifest round-trip through the real Rust renderer.
 $testDir=Join-Path $script:root 'preview/studio-test-themes/qa-canvas'
 New-Item -ItemType Directory -Path $testDir -Force|Out-Null
 Save-Bitmap $null (Join-Path $testDir 'art.png')
 [IO.File]::WriteAllText((Join-Path $testDir 'theme.json'),(@{name='QA Canvas';background='art.png';elements=@($script:elements.ToArray())}|ConvertTo-Json -Depth 8 -Compress))
 $oldThemes=$env:PRC_THEME_DIR;$env:PRC_THEME_DIR=Split-Path $testDir
 & $script:binary qa-canvas | Out-Null
 $env:PRC_THEME_DIR=$oldThemes
 if($LASTEXITCODE -ne 0 -or -not(Test-Path (Join-Path $script:root 'preview/qa-canvas.png'))){throw 'Custom manifest render failed.'}
 while($script:elements.Count -gt 3){$script:elements.RemoveAt($script:elements.Count-1)};Draw-Design
 # Crop bounds: landscape, portrait, tiny source and zoomed edge.
 foreach($spec in @(@(900,450),@(450,900),@(4,3))) {
  $source=New-Object Windows.Media.Imaging.WriteableBitmap($spec[0],$spec[1],96,96,[Windows.Media.PixelFormats]::Bgra32,$null)
  foreach($zoom in @(1,5)) {
   $w=[Math]::Min($spec[0],$spec[1]*4/3)/$zoom;$h=$w*3/4
   $rect=New-Object Windows.Int32Rect(0,0,[int][Math]::Max(1,[Math]::Floor($w)),[int][Math]::Max(1,[Math]::Floor($h)))
   $crop=New-Object Windows.Media.Imaging.CroppedBitmap($source,$rect)
   Save-Bitmap $crop (Join-Path $script:root 'preview/crop-test.png')
  }
 }
 $script:window.Show();$script:window.UpdateLayout()
 foreach($page in @('Themes','Editor','Settings')){Navigate $page;$script:window.UpdateLayout();$target=New-Object Windows.Media.Imaging.RenderTargetBitmap([int]$script:window.ActualWidth,[int]$script:window.ActualHeight,96,96,[Windows.Media.PixelFormats]::Pbgra32);$target.Render($script:window);$encoder=New-Object Windows.Media.Imaging.PngBitmapEncoder;$encoder.Frames.Add([Windows.Media.Imaging.BitmapFrame]::Create($target));$stream=[IO.File]::Create((Join-Path $script:root ('preview/studio-'+$page+'.png')));$encoder.Save($stream);$stream.Dispose()}
 $script:window.Width=1050;$script:window.Height=700;Navigate 'Editor';$script:window.UpdateLayout()
 if($script:ui.DesignCanvas.ActualWidth -ne 320 -or $script:ui.DesignCanvas.ActualHeight -ne 240){throw 'Canvas geometry changed at smaller size.'}
 if($script:ui.ImportImage.ActualWidth -le 0 -or $script:ui.PaletteSection.ActualWidth -le 0){throw 'Editor controls collapsed.'}
 $script:dirty=$false;$script:window.Close();Write-Output 'Studio checks passed: library, previews, palette, invalid extents, rendered pages.';exit
}
$script:window.Dispatcher.Add_UnhandledException({param($sender,$eventArgs)$eventArgs.Handled=$true;Notice ('Action failed: '+$eventArgs.Exception.Message);Add-Content (Join-Path $script:root 'studio-error.log') $eventArgs.Exception.ToString()})
$script:window.Add_ContentRendered({Safe-Action {Load-Faces};if($Editor){Navigate 'Editor'};$script:timer.Start()})
try{$script:window.ShowDialog()|Out-Null}finally{$script:timer.Stop()}
