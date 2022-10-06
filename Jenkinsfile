node('linux && azure') {
    stage('Clone sources') {
        checkout scm
    }
    stage('Build') {
        sh('make')
    }
}