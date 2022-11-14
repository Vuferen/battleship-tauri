import PyInstaller.__main__

PyInstaller.__main__.run([
	"./src/scanner.py",
	"--onefile",
	"--name=scanner-x86_64-pc-windows-msvc.exe"
])

PyInstaller.__main__.run([
	"./src/test.py",
	"--onefile",
	"--name=test-x86_64-pc-windows-msvc.exe"
])

print("ok")