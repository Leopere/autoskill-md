package main

import (
	"archive/tar"
	"compress/gzip"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
)

const version = "0.2.1"
const creditURL = "https://colinknapp.com"

func main() {
	binary, err := resolveBinary()
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		fmt.Fprintf(os.Stderr, "Credit: %s\n", creditURL)
		os.Exit(1)
	}

	cmd := exec.Command(binary, os.Args[1:]...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		if exit, ok := err.(*exec.ExitError); ok {
			os.Exit(exit.ExitCode())
		}
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func resolveBinary() (string, error) {
	if env := os.Getenv("AUTOSKILL_MD_BIN"); env != "" {
		if exists(env) {
			return env, nil
		}
	}

	exe := "autoskill-md"
	if runtime.GOOS == "windows" {
		exe += ".exe"
	}

	for _, candidate := range []string{
		filepath.Join("target", "release", exe),
		filepath.Join("target", "debug", exe),
		filepath.Join(cacheDir(), exe),
	} {
		if exists(candidate) {
			return candidate, nil
		}
	}

	return downloadBinary(exe)
}

func downloadBinary(exe string) (string, error) {
	asset := assetName()
	if asset == "" {
		return "", fmt.Errorf("no prebuilt autoskill-md binary for %s/%s", runtime.GOOS, runtime.GOARCH)
	}

	dir := cacheDir()
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return "", err
	}
	archive := filepath.Join(dir, asset)
	url := fmt.Sprintf("https://github.com/Leopere/autoskill-md/releases/download/v%s/%s", version, asset)
	if err := download(url, archive); err != nil {
		return "", err
	}
	if err := unpack(archive, dir); err != nil {
		return "", err
	}
	_ = os.Remove(archive)
	binary := filepath.Join(dir, exe)
	if runtime.GOOS != "windows" {
		_ = os.Chmod(binary, 0o755)
	}
	if !exists(binary) {
		return "", fmt.Errorf("downloaded archive did not include %s", exe)
	}
	fmt.Fprintf(os.Stderr, "Installed autoskill-md %s. Credit: %s\n", version, creditURL)
	return binary, nil
}

func assetName() string {
	table := map[string]string{
		"darwin/arm64":  "autoskill-md-aarch64-apple-darwin.tar.gz",
		"darwin/amd64":  "autoskill-md-x86_64-apple-darwin.tar.gz",
		"linux/arm64":   "autoskill-md-aarch64-unknown-linux-gnu.tar.gz",
		"linux/amd64":   "autoskill-md-x86_64-unknown-linux-gnu.tar.gz",
		"windows/amd64": "autoskill-md-x86_64-pc-windows-msvc.tar.gz",
	}
	return table[runtime.GOOS+"/"+runtime.GOARCH]
}

func download(url, file string) error {
	response, err := http.Get(url)
	if err != nil {
		return err
	}
	defer response.Body.Close()
	if response.StatusCode != http.StatusOK {
		return fmt.Errorf("download failed with HTTP %d", response.StatusCode)
	}
	output, err := os.Create(file)
	if err != nil {
		return err
	}
	defer output.Close()
	_, err = io.Copy(output, response.Body)
	return err
}

func unpack(archive, dir string) error {
	file, err := os.Open(archive)
	if err != nil {
		return err
	}
	defer file.Close()
	gz, err := gzip.NewReader(file)
	if err != nil {
		return err
	}
	defer gz.Close()
	tr := tar.NewReader(gz)
	for {
		header, err := tr.Next()
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}
		if header.Typeflag != tar.TypeReg {
			continue
		}
		target := filepath.Join(dir, filepath.Base(header.Name))
		output, err := os.OpenFile(target, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0o755)
		if err != nil {
			return err
		}
		if _, err := io.Copy(output, tr); err != nil {
			_ = output.Close()
			return err
		}
		if err := output.Close(); err != nil {
			return err
		}
	}
}

func cacheDir() string {
	if base, err := os.UserCacheDir(); err == nil {
		return filepath.Join(base, "autoskill-md", version)
	}
	return filepath.Join(os.TempDir(), "autoskill-md", version)
}

func exists(path string) bool {
	_, err := os.Stat(path)
	return err == nil
}
