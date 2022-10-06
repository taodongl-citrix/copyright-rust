import com.atlassian.bitbucket.event.pull.PullRequestOpenedEvent
import com.atlassian.bitbucket.event.pull.PullRequestRescopedEvent
import com.atlassian.bitbucket.event.pull.PullRequestReopenedEvent

import groovy.transform.Field
import java.nio.charset.StandardCharsets
import org.apache.http.client.config.RequestConfig
import org.apache.http.client.methods.HttpPost
import org.apache.http.entity.ContentType
import org.apache.http.entity.StringEntity
import org.apache.http.impl.client.HttpClientBuilder
import com.google.common.hash.Hashing
import groovy.json.JsonOutput

@Field def SECRET = "0143207be7c417eb8444a552d78b61deffa64efd"

if (event instanceof PullRequestOpenedEvent || event instanceof PullRequestRescopedEvent || event instanceof PullRequestReopenedEvent) {
    def trigger = new ScannerTrigger(event.pullRequest, SECRET)
    trigger.execute()
}

class ScannerTrigger {
    String payload
    String signature
    ScannerTrigger(def pullRequest, def secret) {
        def req = [
            id: pullRequest.id,
            from: [
                project: pullRequest.fromRef.repository.project.key,
                repository: pullRequest.fromRef.repository.slug,
                branch: pullRequest.fromRef.displayId,
                commit: pullRequest.fromRef.latestCommit
            ],
            to: [
                project: pullRequest.toRef.repository.project.key,
                repository: pullRequest.toRef.repository.slug,
                branch: pullRequest.toRef.displayId,
                commit: pullRequest.toRef.latestCommit
            ]
        ]
        this.payload = JsonOutput.toJson(req)
        this.signature = Hashing.hmacSha256(secret.getBytes(StandardCharsets.UTF_8))
                        .hashString(this.payload, StandardCharsets.UTF_8)
                        .toString()
    }

    void execute() {
        def baseUrl = "https://smee.io/6cexv0ReEvNfdgnz"
        //def baseUrl = "http://10.158.160.26/api/trigger"
        def reqTimeout = 10000
        RequestConfig reqConfig = RequestConfig.custom()
                .setConnectionRequestTimeout(reqTimeout)
                .setConnectTimeout(reqTimeout)
                .setSocketTimeout(reqTimeout)
                .build();
        HttpClientBuilder.create().setDefaultRequestConfig(reqConfig).build().withCloseable { client ->
            def httpPost = new HttpPost(baseUrl)
            def entity = new StringEntity(this.payload, ContentType.APPLICATION_JSON)
            httpPost.setEntity(entity);
            httpPost.addHeader("X-Hub-Signature-256", "sha256=" + this.signature)
            client.execute(httpPost).withCloseable { response ->
                int code = response.getStatusLine().getStatusCode()
                if (code < 200 || code >399) {
                    throw new RuntimeException("fail to access: " + baseUrl + ", status: " + code)
                }
            }
        }
    }
}
