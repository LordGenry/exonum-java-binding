/*
 * Copyright 2018 The Exonum Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
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

package com.exonum.binding.common.message;

import static com.exonum.binding.common.message.ByteBufferAllocator.allocateBuffer;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

import com.exonum.binding.common.crypto.CryptoFunction;
import com.exonum.binding.common.crypto.CryptoFunctions;
import com.exonum.binding.common.crypto.KeyPair;
import com.exonum.binding.common.crypto.PrivateKey;
import com.exonum.binding.common.crypto.PublicKey;
import com.exonum.binding.common.hash.HashCode;
import com.exonum.binding.common.hash.Hashing;
import com.exonum.binding.test.Bytes;
import org.junit.jupiter.api.Test;

class BinaryMessageTest {

  private static final Message MESSAGE_TEMPLATE = new Message.Builder()
      .setNetworkId((byte) 0x01)
      .setVersion((byte) 0x02)
      .setServiceId((short) 0xA103)
      .setMessageType((short) 0xB204)
      .setBody(allocateBuffer(2))
      .setSignature(new byte[64])
      .build();

  @Test
  void getMessageNoSignature() {
    BinaryMessage message = new Message.Builder()
        .setNetworkId((byte) 0x01)
        .setVersion((byte) 0x02)
        .setServiceId((short) 0xA103)
        .setMessageType((short) 0xB204)
        .setBody(allocateBuffer(2))
        .setSignature(new byte[64])
        .buildRaw();

    byte[] expectedNoSignature = Bytes.fromHex("010204b203a14c0000000000");
    assertThat(message.getMessageNoSignature(), equalTo(expectedNoSignature));
  }

  @Test
  void hash() {
    BinaryMessage message = new Message.Builder()
        .mergeFrom(MESSAGE_TEMPLATE)
        .buildRaw();

    HashCode hash = message.hash();

    assertThat(hash.bits(), equalTo(Hashing.DEFAULT_HASH_SIZE_BITS));
  }

  @Test
  void verifyValid() {
    BinaryMessage message = new Message.Builder()
        .mergeFrom(MESSAGE_TEMPLATE)
        .buildRaw();

    PublicKey publicKey = PublicKey.fromHexString("ab");
    CryptoFunction cf = mock(CryptoFunction.class);
    when(cf.verify(eq(message.getMessageNoSignature()), eq(message.getSignature()), eq(publicKey)))
        .thenReturn(true);

    assertTrue(message.verify(cf, publicKey));
  }

  @Test
  void verifyInvalid() {
    BinaryMessage message = new Message.Builder()
        .mergeFrom(MESSAGE_TEMPLATE)
        .buildRaw();

    PublicKey publicKey = PublicKey.fromHexString("ab");
    CryptoFunction cf = mock(CryptoFunction.class);
    when(cf.verify(eq(message.getMessageNoSignature()), eq(message.getSignature()), eq(publicKey)))
        .thenReturn(false);

    assertFalse(message.verify(cf, publicKey));
  }

  @Test
  void sign() {
    byte[] expectedSignature = Bytes.createPrefixed(Bytes.bytes(0x0A, 0x0B),
        Message.SIGNATURE_SIZE);

    BinaryMessage unsignedMessage = new Message.Builder()
        .mergeFrom(MESSAGE_TEMPLATE)
        .buildRaw();

    PrivateKey privateKey = PrivateKey.fromHexString("cd");
    CryptoFunction cf = mock(CryptoFunction.class);
    when(cf.signMessage(eq(unsignedMessage.getMessageNoSignature()), eq(privateKey)))
        .thenReturn(expectedSignature);

    BinaryMessage signedMessage = unsignedMessage.sign(cf, privateKey);

    assertThat(signedMessage.getSignature(), equalTo(expectedSignature));
    verify(cf).signMessage(any(), any());
  }

  @Test
  void signVerifyRoundtripIntegrationTest() {
    CryptoFunction cf = CryptoFunctions.ed25519();
    KeyPair keyPair = cf.generateKeyPair();

    BinaryMessage unsignedMessage = new Message.Builder()
        .mergeFrom(MESSAGE_TEMPLATE)
        .buildRaw();

    BinaryMessage signedMessage = unsignedMessage.sign(cf, keyPair.getPrivateKey());

    assertTrue(signedMessage.verify(cf, keyPair.getPublicKey()));
  }
}
