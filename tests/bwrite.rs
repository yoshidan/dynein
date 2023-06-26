/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License").
 * You may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

pub mod util;

use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::fs::File;
use std::io::Write;
use tempfile::Builder;

#[tokio::test]
async fn test_batch_write() -> Result<(), Box<dyn std::error::Error>> {
    let table_name = "table--test_batch_write";

    let mut c = util::setup().await?;
    c.args(&[
        "--region", "local", "admin", "create", "table", table_name, "--keys", "pk",
    ])
    .output()?;

    let tmpdir = Builder::new().tempdir()?; // defining stand alone variable here as tempfile::tempdir creates directory and deletes it when the destructor is run.
    let batch_input_file_path = tmpdir.path().join("test_batch_write.json");
    let mut f = File::create(tmpdir.path().join("test_batch_write.json"))?;
    f.write_all(b"
     {
         \"table--test_batch_write\": [
             {
                 \"PutRequest\": {
                     \"Item\": {
                         \"pk\": { \"S\": \"ichi\" },
                         \"ISBN\": { \"S\": \"111-1111111111\" },
                         \"Price\": { \"N\": \"2\" },
                         \"Dimensions\": { \"SS\": [\"Giraffe\", \"Hippo\" ,\"Zebra\"] },
                         \"PageCount\": { \"NS\": [\"42.2\", \"-19\", \"7.5\", \"3.14\"] },
                         \"InPublication\": { \"BOOL\": false },
                         \"Nothing\": { \"NULL\": true },
                         \"Authors\": {
                             \"L\": [
                                 { \"S\": \"Author1\" },
                                 { \"S\": \"Author2\" },
                                 { \"N\": \"42\" }
                             ]
                         },
                         \"Details\": {
                             \"M\": {
                                 \"Name\": { \"S\": \"Joe\" },
                                 \"Age\":  { \"N\": \"35\" },
                                 \"Misc\": {
                                     \"M\": {
                                         \"hope\": { \"BOOL\": true },
                                         \"dream\": { \"L\": [ { \"N\": \"35\" }, { \"NULL\": true } ] }
                                     }
                                 }
                             }
                         }
                     }
                 }
             }
         ]
     }
     ")?;
    let mut c = util::setup().await?;
    c.args(&[
        "--region",
        "local",
        "--table",
        table_name,
        "bwrite",
        "--input",
        batch_input_file_path.to_str().unwrap(),
    ])
    .output()?;

    let mut c = util::setup().await?;
    let scan_cmd = c.args(&["--region", "local", "--table", table_name, "scan"]);
    scan_cmd
        .assert()
        .success()
        .stdout(predicate::str::is_match("pk *attributes\nichi").unwrap());

    /*
    get output should looks like:
        $ dy --region local -t table--test_batch_write get ichi
        {
          "Dimensions": [
            "Giraffe",
            "Hippo",
            "Zebra"
          ],
          "PageCount": [
            -19.0,
            3.14,
            7.5,
            42.2
          ],
          "Authors": [
            "Author1",
            "Author2",
            42
          ],
          "InPublication": false,
          "Nothing": null,
          "Price": 2,
          "pk": "ichi",
          "Details": {
            "Age": 35,
            "Misc": {
              "dream": [
                35,
                null
              ],
              "hope": true
            },
            "Name": "Joe"
          },
          "ISBN": "111-1111111111"
        }
    */
    let mut c = util::setup().await?;
    let get_cmd = c.args(&["--region", "local", "--table", table_name, "get", "ichi"]);
    let output = get_cmd.output()?.stdout;

    // more verification would be nice
    assert_eq!(
        true,
        predicate::str::is_match("\"Dimensions\":")?.eval(String::from_utf8(output)?.as_str())
    );

    util::cleanup(vec![table_name]).await
}