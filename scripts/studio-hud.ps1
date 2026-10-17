function Normalize-Element($e) {
 $defaults=@{label=$null;unit=$null;text='Custom text';show_label=$true;show_value=$true;show_unit=$true;label_size=12;value_color=@(245,248,255);label_color=@($e.color);track_color=@(43,57,65);background_color=@(0,0,0);opacity=0;stroke=5;decimals=0}
 foreach($key in $defaults.Keys){if(-not $e.ContainsKey($key)){$e[$key]=$defaults[$key]}}
 if($null -eq $e.label_color){$e.label_color=@($e.color)}
 return $e
}
function Fit-Text($panel,$text,$size,$colour,$x,$y,$w,$center=$false) {
 $block=Text-Visual $text $size $colour
 $block.Measure((New-Object Windows.Size([double]::PositiveInfinity,[double]::PositiveInfinity)))
 while($block.DesiredSize.Width -gt $w -and $block.FontSize -gt 8){$block.FontSize--; $block.Measure((New-Object Windows.Size([double]::PositiveInfinity,[double]::PositiveInfinity)))}
 $block.Width=$w;if($center){$block.TextAlignment='Center'}
 [Windows.Controls.Canvas]::SetLeft($block,$x);[Windows.Controls.Canvas]::SetTop($block,$y);$panel.Children.Add($block)|Out-Null
}
function Element-Visual($e){
 $e=Normalize-Element $e
 $panel=New-Object Windows.Controls.Canvas;$panel.Width=$e.width;$panel.Height=$e.height;$panel.ClipToBounds=$true
 $sensor=Sensor $e.sensor;$accent=Brush $e.color;$ink=Brush $e.value_color;$labelInk=Brush $e.label_color;$trackInk=Brush $e.track_color
 $label=if($null -eq $e.label){$e.sensor.ToUpper().Replace('.',' ')}else{$e.label}
 $suffix=if($null -eq $e.unit){$sensor.unit}else{$e.unit}

 if(-not $e.show_unit){$suffix=''}
 $value=if($sensor){([double]$sensor.demo).ToString(('F'+[int]$e.decimals),[Globalization.CultureInfo]::InvariantCulture)+$suffix}else{'—'}
 $background=New-Object Windows.Controls.Border;$background.Width=$e.width;$background.Height=$e.height;$background.CornerRadius=5;$background.Background=Brush $e.background_color;$background.Opacity=[double]$e.opacity/255;$panel.Children.Add($background)|Out-Null
 if($e.kind -eq 'panel'){return $panel}
 $pad=4;$w=[Math]::Max(4,$e.width-8);$y=4
 if($e.kind -eq 'label'){Fit-Text $panel $e.text $e.size $ink 4 4 $w;return $panel}
 if($e.kind -eq 'ring'){
  $radius=[Math]::Max(1,([Math]::Min($e.width,$e.height)/2-$e.stroke/2-4));$cx=$e.width/2;$cy=$e.height/2
  $fraction=if($sensor){[Math]::Max(0.001,[Math]::Min(1.0,[double]$sensor.demo/[double]$e.max))}else{0}
  foreach($spec in @(@{fraction=1;color=$trackInk},@{fraction=$fraction;color=$accent})){
   if($spec.fraction -le 0){continue}
   $start=135*[Math]::PI/180;$end=(135+270*$spec.fraction)*[Math]::PI/180
   $fig=New-Object Windows.Media.PathFigure;$fig.StartPoint=New-Object Windows.Point(($cx+$radius*[Math]::Cos($start)),($cy+$radius*[Math]::Sin($start)))
   $seg=New-Object Windows.Media.ArcSegment;$seg.Point=New-Object Windows.Point(($cx+$radius*[Math]::Cos($end)),($cy+$radius*[Math]::Sin($end)));$seg.Size=New-Object Windows.Size($radius,$radius);$seg.IsLargeArc=($spec.fraction*270 -gt 180);$seg.SweepDirection='Clockwise';$fig.Segments.Add($seg)
   $geo=New-Object Windows.Media.PathGeometry;$geo.Figures.Add($fig);$path=New-Object Windows.Shapes.Path;$path.Data=$geo;$path.Stroke=$spec.color;$path.StrokeThickness=$e.stroke;$path.StrokeStartLineCap='Round';$path.StrokeEndLineCap='Round';$panel.Children.Add($path)|Out-Null
  }
  if($e.show_label){Fit-Text $panel $label $e.label_size $labelInk 4 ($cy-$e.label_size-8) $w $true}
  if($e.show_value){Fit-Text $panel $value $e.size $ink 4 ($cy-2) $w $true}
  return $panel
 }
 if($e.show_label){Fit-Text $panel $label $e.label_size $labelInk 4 $y $w;$y+=$e.label_size+2}
 if($e.show_value){Fit-Text $panel $value $e.size $ink 4 $y $w;$y+=$e.size+3}
 $h=[Math]::Max(0,$e.height-4-$y)
 if($e.kind -eq 'bar' -and $h -ge 3){
  $track=New-Object Windows.Shapes.Rectangle;$track.Width=$w;$track.Height=$h;$track.RadiusX=4;$track.RadiusY=4;$track.Fill=$trackInk;[Windows.Controls.Canvas]::SetTop($track,$y);[Windows.Controls.Canvas]::SetLeft($track,4);$panel.Children.Add($track)|Out-Null
  $fill=New-Object Windows.Shapes.Rectangle;$fill.Width=$w*[Math]::Max(0.0,[Math]::Min(1.0,[double]$sensor.demo/[double]$e.max));$fill.Height=$h;$fill.RadiusX=4;$fill.RadiusY=4;$fill.Fill=$accent;[Windows.Controls.Canvas]::SetTop($fill,$y);[Windows.Controls.Canvas]::SetLeft($fill,4);$panel.Children.Add($fill)|Out-Null
 }
 if($e.kind -eq 'graph' -and $h -ge 3){
  for($i=0;$i -lt 3;$i++){$line=New-Object Windows.Shapes.Line;$line.X1=4;$line.X2=4+$w;$line.Y1=$y+$h*$i/2;$line.Y2=$line.Y1;$line.Stroke=$trackInk;$line.StrokeThickness=1;$panel.Children.Add($line)|Out-Null}
  $path=New-Object Windows.Shapes.Polyline;$path.Stroke=$accent;$path.StrokeThickness=[Math]::Min(6,$e.stroke)
  for($i=0;$i -lt 40;$i++){$fraction=0.45+0.12*[Math]::Sin($i*0.7)+0.05*[Math]::Cos($i*1.9);$path.Points.Add((New-Object Windows.Point((4+$w*$i/39),($y+$h*(1-$fraction)))))}
  $panel.Children.Add($path)|Out-Null
 }
 return $panel
}
function Update-Inspector {
 $script:updating=$true
 if($script:selectedElement -ge 0 -and $script:selectedElement -lt $script:elements.Count){
  $e=Normalize-Element $script:elements[$script:selectedElement]
  foreach($pair in @(@('PropX','x'),@('PropY','y'),@('PropWidth','width'),@('PropHeight','height'),@('PropSize','size'),@('PropMax','max'),@('PropLabelSize','label_size'),@('PropStroke','stroke'),@('PropDecimals','decimals'))){$script:ui[$pair[0]].Text=[string]$e[$pair[1]]}
  $script:ui.SensorSelect.SelectedValue=$e.sensor;$script:ui.ModeSelect.SelectedValue=$e.kind
  $script:ui.PropLabel.Text=[string]$e.label;$script:ui.PropUnit.Text=[string]$e.unit;$script:ui.PropText.Text=$e.text
  $script:ui.ShowLabel.IsChecked=$e.show_label;$script:ui.ShowValue.IsChecked=$e.show_value;$script:ui.ShowUnit.IsChecked=$e.show_unit;$script:ui.PropOpacity.Value=[Math]::Round($e.opacity*100/255)
 }
 foreach($key in @('ApplyProperties','ApplyColour','DeleteLayer','ModeSelect','PropLabel','PropUnit','PropText','ShowLabel','ShowValue','ShowUnit')){$script:ui[$key].IsEnabled=($script:selectedElement -ge 0)}
 $script:updating=$false;Update-Colour
}
function Update-Colour($key) {
 if($script:selectedElement -lt 0){return}
 $e=Normalize-Element $script:elements[$script:selectedElement];if(-not $key){$key=$script:ui.ColourTarget.SelectedValue}
 if($key -and $e[$key]){$rgb=$e[$key];$script:ui.ColourHex.Text=('#{0:X2}{1:X2}{2:X2}' -f [int]$rgb[0],[int]$rgb[1],[int]$rgb[2])}
}
function Apply-Colour($rgb) {
 if($script:selectedElement -lt 0){throw 'Select a layer first.'}
 $key=$script:ui.ColourTarget.SelectedValue;if(-not $key){$key='color'}
 $script:elements[$script:selectedElement][$key]=@($rgb);$script:dirty=$true;Draw-Design -PreserveInspector;Update-Colour
}
function Add-Element($kind){
 if($script:elements.Count -ge 64){throw 'A face can contain up to 64 HUD elements.'}
 $sensor=$script:ui.AddSensor.SelectedItem
 $e=@{kind=$kind;sensor=$sensor.id;x=20;y=65;width=110;height=110;size=28;max=$sensor.max;color=@(0,195,255)}
 $e=Normalize-Element $e;$e.opacity=0;$e.show_label=$false
 switch($kind){'bar'{$e.width=190;$e.height=75;$e.size=22};'text'{$e.height=70};'graph'{$e.width=250;$e.height=100;$e.size=22;$e.stroke=2};'label'{$e.width=180;$e.height=50;$e.text='My PC';$e.show_label=$false;$e.show_value=$false};'panel'{$e.opacity=166;$e.width=250;$e.height=150;$e.show_label=$false;$e.show_value=$false}}
 # Find a free initial location instead of stacking all new elements.
 $found=$false
 for($y=8;$y -le 240-$e.height -and -not $found;$y+=8){for($x=8;$x -le 320-$e.width;$x+=8){$overlap=$false;foreach($other in $script:elements){if($other.kind -eq 'panel'){continue};if($x -lt $other.x+$other.width -and $x+$e.width -gt $other.x -and $y -lt $other.y+$other.height -and $y+$e.height -gt $other.y){$overlap=$true;break}};if(-not $overlap){$e.x=$x;$e.y=$y;$found=$true;break}}}
 $script:elements.Add($e)|Out-Null;$script:selectedElement=$script:elements.Count-1;$script:dirty=$true;Draw-Design
 $script:ui.PaletteSection.IsExpanded=$true
 Notice $(if($found){'HUD added. Resize with its corner handle or use the inspector.'}else{'HUD added. Canvas is crowded; move it to avoid overlapping another layer.'})
}
function Element-Valid($e) {
 $e=Normalize-Element $e
 foreach($key in @('x','y','width','height','size','max','label_size','stroke','decimals')){if([double]::IsNaN($e[$key]) -or [double]::IsInfinity($e[$key])){return $false}}
 return ($e.x -ge 0 -and $e.y -ge 0 -and $e.width -ge 12 -and $e.height -ge 12 -and $e.x+$e.width -le 320 -and $e.y+$e.height -le 240 -and $e.size -ge 8 -and $e.size -le 72 -and $e.size -eq [Math]::Floor($e.size) -and $e.label_size -ge 8 -and $e.label_size -le 32 -and $e.label_size -eq [Math]::Floor($e.label_size) -and $e.max -gt 0 -and $e.stroke -ge 1 -and $e.stroke -le 20 -and $e.decimals -ge 0 -and $e.decimals -le 3 -and $e.decimals -eq [Math]::Floor($e.decimals) -and $e.opacity -ge 0 -and $e.opacity -le 255)
}
function Apply-Properties {
 if($script:selectedElement -lt 0){throw 'Select a layer first.'};$e=(Normalize-Element $script:elements[$script:selectedElement]).Clone()
 foreach($pair in @(@('PropX','x'),@('PropY','y'),@('PropWidth','width'),@('PropHeight','height'),@('PropSize','size'),@('PropMax','max'),@('PropLabelSize','label_size'),@('PropStroke','stroke'),@('PropDecimals','decimals'))){$n=0.0;if(-not [double]::TryParse($script:ui[$pair[0]].Text,[ref]$n)){throw 'Use valid numbers in the inspector.'};$e[$pair[1]]=$n}
 $e.sensor=$script:ui.SensorSelect.SelectedValue;$e.kind=$script:ui.ModeSelect.SelectedValue
 $e.label=if($script:ui.PropLabel.Text){$script:ui.PropLabel.Text}else{$null};$e.unit=if($script:ui.PropUnit.Text){$script:ui.PropUnit.Text}else{$null};$e.text=$script:ui.PropText.Text
 $e.show_label=[bool]$script:ui.ShowLabel.IsChecked;$e.show_value=[bool]$script:ui.ShowValue.IsChecked;$e.show_unit=[bool]$script:ui.ShowUnit.IsChecked;$e.opacity=[int][Math]::Round($script:ui.PropOpacity.Value*255/100)
 if(-not(Element-Valid $e)){throw 'Keep HUD inside 320 × 240; fonts 8–72 (label 8–32), thickness 1–20, decimals 0–3, maximum above zero.'}
 # Integer serde fields must remain integers in saved manifests.
 foreach($key in @('size','label_size','decimals','opacity')){$e[$key]=[int]$e[$key]}
 $script:elements[$script:selectedElement]=$e;$script:dirty=$true;Draw-Design;Notice 'HUD updated.'
}
