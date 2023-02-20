use std::{path::Path, process::Command};

use anyhow::Result;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;

const TEST_BAR_CODE_PATH: &str = "tests/images/qr-code.png";
const TEST_BAR_CODE_CONTENT: &str = "QR-Code:https://github.com/mchehab/zbar\n";
const TEST_MULTIPLE_BAR_CODE_PATH: &str = "tests/images/multiple.png";
const TEST_MULTIPLE_BAR_CODE_CONTENT: &str = "EAN-13:9789876543217\nCodabar:A9876543210B\n";
const TEST_NO_BAR_CODE_PATH: &str = "tests/images/no-code.png";

#[test]
fn should_fail_if_no_image_provided() -> Result<()> {
    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.assert().failure();

    Ok(())
}

#[test]
fn should_fail_if_file_doesnt_exist() -> Result<()> {
    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg("test/file/doesnt/exist");
    cmd.assert().failure();

    Ok(())
}

#[test]
fn should_fail_if_file_is_no_image() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("no_image.txt")?;

    file.write_str("This file is not an image")?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Failed to open image"));

    Ok(())
}

#[test]
fn should_return_type_and_data_for_scanned_image() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("barcode.png")?;

    file.write_file(Path::new(TEST_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::eq(TEST_BAR_CODE_CONTENT));

    Ok(())
}

#[test]
fn should_return_types_and_datas_for_scanned_image_with_multiple_codes() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("multiple.png")?;

    file.write_file(Path::new(TEST_MULTIPLE_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path());
    cmd.assert()
        .success()
        .stdout(predicate::eq(TEST_MULTIPLE_BAR_CODE_CONTENT));

    Ok(())
}

#[test]
fn should_fail_when_scanned_image_has_no_code() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("no-code.png")?;

    file.write_file(Path::new(TEST_NO_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path());
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No symbol detected"));

    Ok(())
}

#[test]
fn should_return_types_and_datas_for_two_scanned_images() -> Result<()> {
    let single_code_file = assert_fs::NamedTempFile::new("barcode.png")?;
    let multiple_codes_file = assert_fs::NamedTempFile::new("multiple.png")?;

    single_code_file.write_file(Path::new(TEST_BAR_CODE_PATH))?;
    multiple_codes_file.write_file(Path::new(TEST_MULTIPLE_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(single_code_file.path())
        .arg(multiple_codes_file.path());
    cmd.assert().success().stdout(predicate::eq(
        format!("{TEST_BAR_CODE_CONTENT}{TEST_MULTIPLE_BAR_CODE_CONTENT}").as_str(),
    ));

    Ok(())
}

#[test]
fn should_return_xml_for_scanned_image_when_passing_xml_flag() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("barcode.png")?;

    file.write_file(Path::new(TEST_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path()).arg("--xml");
    cmd.assert()
        .success()
        .stdout(predicate::eq(format!("<barcodes xmlns=\"http://zbar.sourceforge.net/2008/barcode\">\n    \
            <source href=\"{}\">\n        \
                <index num=\"0\">\n            \
                    <symbol type=\'QR-Code\' quality=\'1\' orientation=\'UP\'><polygon points=\'+1,+1 +0,+98 +100,+100 +98,+0\'/><data><![CDATA[https://github.com/mchehab/zbar]]></data></symbol>\n        \
                </index>\n    \
            </source>\n\
        </barcodes>\n", file.path().display()).as_str()));

    Ok(())
}

#[test]
fn should_return_only_data_for_scanned_image_when_passing_raw_flag() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("barcode.png")?;

    file.write_file(Path::new(TEST_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path()).arg("--raw");
    cmd.assert()
        .success()
        .stdout(predicate::eq("https://github.com/mchehab/zbar\n"));

    Ok(())
}

#[test]
fn should_return_polygon_for_scanned_image_when_passing_polygon_flag() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("barcode.png")?;

    file.write_file(Path::new(TEST_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path()).arg("--polygon");
    cmd.assert().success().stdout(predicate::eq(
        "QR-Code:1,1 0,98 100,100 98,0:https://github.com/mchehab/zbar\n",
    ));

    Ok(())
}

#[test]
fn should_return_single_code_for_scanned_images_when_passing_oneshot_flag() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("barcode.png")?;

    file.write_file(Path::new(TEST_MULTIPLE_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path()).arg("--oneshot");
    cmd.assert()
        .success()
        .stdout(predicate::eq("EAN-13:9789876543217\n"));

    Ok(())
}

#[test]
fn should_fail_to_parse_invalid_config() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("barcode.png")?;

    file.write_file(Path::new(TEST_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path()).arg("--set=test");
    cmd.assert().failure().stderr(predicate::str::contains(
        "Failed to parse the config `test`",
    ));

    Ok(())
}

#[test]
fn should_fail_when_diabling_code_with_config_flag() -> Result<()> {
    let file = assert_fs::NamedTempFile::new("barcode.png")?;

    file.write_file(Path::new(TEST_BAR_CODE_PATH))?;

    let mut cmd = Command::cargo_bin("rsbar-img")?;

    cmd.arg(file.path()).arg("--set=qrcode.disable");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("No symbol detected"));

    Ok(())
}
