# Homebrew Formula for Rosetta - iOS Localization Tool
class Rosetta < Formula
    desc "🌍 Modern iOS localization tool with beautiful CLI"
    homepage "https://github.com/yourusername/rosetta"
    url "https://github.com/yourusername/rosetta/archive/v0.1.0.tar.gz"
    sha256 "YOUR_SHA256_HERE"
    license "MIT"
  
    depends_on "rust" => :build
  
    def install
      system "cargo", "install", *std_cargo_args
    end
  
    test do
      system "#{bin}/rosetta", "--help"
    end
  end
  
  # Installation:
  # 1. Create your own homebrew tap:
  #    brew tap yourusername/tools
  #
  # 2. Place this file in the Formula/ directory of your homebrew-tools repository
  #
  # 3. Users can install using:
  #    brew install yourusername/tools/rosetta
  #
  # 4. Or install directly from local:
  #    brew install --build-from-source ./rosetta.rb 