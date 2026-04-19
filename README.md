# citygml-to-stid

## 使い方
この crate は、3D の面データを空間 ID に変換するためのライブラリです。
PLATEAU の CityGML は一例で、今後は他の入力形式もアダプタとして追加できます。

### PLATEAU を使う例
```rust
use citygml_spatial_id::plateau_building;

let ids = plateau_building(xml_string, 25, 0.0)?;
println!("{}", ids.len());
```