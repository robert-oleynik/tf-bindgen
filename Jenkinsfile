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
					sh 'cargo fmt --check --all'
				}
			}
		}
		stage('Lint') {
			steps {
				container('rust') {
					sh 'rustup component add clippy'
					sh 'cargo clippy --workspace'
				}
			}
		}
		stage('Build') {
			steps {
				container('rust') {
					sh 'cargo build --workspace'
				}
			}
		}
		stage('Test') {
			steps {
				container('rust') {
					sh 'cargo test --workspace --tests'
				}
			}
		}
	}
}
