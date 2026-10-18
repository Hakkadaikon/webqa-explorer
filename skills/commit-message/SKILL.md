---
name: commit-message
description: >
  このリポジトリでコミットメッセージを書くときの規約。
  Conventional Commitsの形で、要約と本文は日本語。
  コミットする前、コミットメッセージを書く・直すとき、
  hymme:micro-commitやhymme:commit-flowでコミットするときに必ず使う。
---

# コミットメッセージ

規約の本体は[`docs/commit-messages.md`](../../docs/commit-messages.md)にある。
コミットする前に必ず読む。

この規約は、hymme:micro-commitの「英語で書く」という既定より優先する。
種類と範囲は英語で、要約と本文は日本語で書く。

## 要点

- 1行目は`<種類>(<範囲>)!: <要約>`の形にする。
- 種類はfeat、fix、docs、style、refactor、perf、test、build、ci、chore、revert。
- 範囲は英小文字・数字・ハイフンで、省いてもよい。
- 要約は日本語で50字以内とし、文末に句点を付けない。
- 要約は現在形で書き、過去形にしない。
- 本文は1行72桁以内で、なぜ変えるのかを書く。
- 管理番号や設計手法の名前は書かない。

## 検査

- コミットのたびにlefthookのcommit-msgフックが判定する。
- 判定に落ちたら、規約に合わせて書き直す。
- `--no-verify`で飛ばさない。
- 手元で試すときは`just commit-msg <ファイル>`を使う。
