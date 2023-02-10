pipeline {
	agent {
		kubernetes {
			yamlFile ".jenkins/pod.yaml"
		}
	}
	stages {
		stage('Format') {
			steps {
				container('rust') {
					sh 'rustup component add rustfmt'
					sh 'cargo fmt --check'
				}
			}
		}
		stage('Lint') {
			steps {
				container('rust') {
					sh 'rustup component add clippy'
					sh 'cargo clippy'
				}
			}
		}
		stage('Build') {
			steps {
				container('rust') {
					sh 'cargo build'
				}
			}
		}
		stage('Test') {
			steps {
				container('rust') {
					sh 'cargo test'
				}
			}
		}
	}
}
